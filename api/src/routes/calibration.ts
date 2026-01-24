import { createRoute, OpenAPIHono, z } from '@hono/zod-openapi';
import { eq } from 'drizzle-orm';

import { patternPrediction, db } from '@/db/index';
import {
	computeCalibrationMetrics,
	detectDrift,
	fitPlattScaling,
	fitIsotonicRegression,
} from '@/lib/calibration';

// Schemas
const CalibrationBinSchema = z.object({
	confidenceRange: z.tuple([z.number(), z.number()]),
	predictions: z.number(),
	correct: z.number(),
	accuracy: z.number(),
});

const CalibrationMetricsSchema = z
	.object({
		bins: z.array(CalibrationBinSchema),
		expectedCalibrationError: z.number(),
		maxCalibrationError: z.number(),
		brierScore: z.number(),
		coverage: z.number(),
		totalPredictions: z.number(),
		validatedPredictions: z.number(),
	})
	.openapi('CalibrationMetrics');

const DriftAlertSchema = z
	.object({
		severity: z.enum(['low', 'medium', 'high']),
		message: z.string(),
		recommendation: z.string(),
		eceChange: z.number(),
		brierChange: z.number(),
	})
	.openapi('DriftAlert');

const ValidatePredictionSchema = z
	.object({
		wasCorrect: z.boolean(),
		validationSource: z
			.enum(['user_feedback', 'mod_author', 'auto'])
			.default('user_feedback'),
	})
	.openapi('ValidatePrediction');

const ErrorSchema = z
	.object({
		error: z.object({
			code: z.string(),
			message: z.string(),
		}),
	})
	.openapi('Error');

// Routes
const getMetricsRoute = createRoute({
	method: 'get',
	path: '/metrics',
	tags: ['Calibration'],
	summary: 'Get calibration metrics',
	description: 'Get current calibration metrics for pattern predictions',
	request: {
		query: z.object({
			days: z.coerce.number().min(1).max(365).default(30),
		}),
	},
	responses: {
		200: {
			content: {
				'application/json': {
					schema: CalibrationMetricsSchema,
				},
			},
			description: 'Calibration metrics',
		},
	},
});

const checkDriftRoute = createRoute({
	method: 'get',
	path: '/drift',
	tags: ['Calibration'],
	summary: 'Check for calibration drift',
	description: 'Compare recent calibration to baseline and detect drift',
	request: {
		query: z.object({
			recentDays: z.coerce.number().min(1).max(30).default(7),
			baselineDays: z.coerce.number().min(7).max(365).default(90),
		}),
	},
	responses: {
		200: {
			content: {
				'application/json': {
					schema: z.object({
						hasDrift: z.boolean(),
						alert: DriftAlertSchema.nullable(),
						recent: CalibrationMetricsSchema,
						baseline: CalibrationMetricsSchema,
					}),
				},
			},
			description: 'Drift check results',
		},
	},
});

const validatePredictionRoute = createRoute({
	method: 'post',
	path: '/predictions/{id}/validate',
	tags: ['Calibration'],
	summary: 'Validate a prediction',
	description: 'Submit validation for a pattern prediction',
	request: {
		params: z.object({
			id: z.string(),
		}),
		body: {
			content: {
				'application/json': {
					schema: ValidatePredictionSchema,
				},
			},
			required: true,
		},
	},
	responses: {
		200: {
			content: {
				'application/json': {
					schema: z.object({
						success: z.boolean(),
					}),
				},
			},
			description: 'Validation recorded',
		},
		404: {
			content: {
				'application/json': {
					schema: ErrorSchema,
				},
			},
			description: 'Prediction not found',
		},
	},
});

const recalibrateRoute = createRoute({
	method: 'post',
	path: '/recalibrate',
	tags: ['Calibration'],
	summary: 'Recalibrate confidence scores',
	description: 'Fit a recalibration model from validated predictions',
	request: {
		body: {
			content: {
				'application/json': {
					schema: z.object({
						method: z.enum(['platt', 'isotonic']).default('platt'),
						days: z.number().min(7).max(365).default(90),
					}),
				},
			},
			required: true,
		},
	},
	responses: {
		200: {
			content: {
				'application/json': {
					schema: z.object({
						method: z.string(),
						samplesUsed: z.number(),
						parameters: z.any(),
					}),
				},
			},
			description: 'Recalibration complete',
		},
	},
});

// App and handlers
const calibrationApp = new OpenAPIHono();

calibrationApp.openapi(getMetricsRoute, async (c) => {
	const { days } = c.req.valid('query');

	const since = new Date(Date.now() - days * 24 * 60 * 60 * 1000);

	const predictions = await db.query.patternPrediction.findMany({
		where: (p, { gte }) => gte(p.createdAt, since),
	});

	const metrics = computeCalibrationMetrics(
		predictions.map((p) => ({
			id: p.id,
			predictedConfidence: p.predictedConfidence,
			wasCorrect: p.wasCorrect,
			validatedAt: p.validatedAt,
		})),
	);

	return c.json(metrics);
});

calibrationApp.openapi(checkDriftRoute, async (c) => {
	const { recentDays, baselineDays } = c.req.valid('query');

	const now = Date.now();
	const recentSince = new Date(now - recentDays * 24 * 60 * 60 * 1000);
	const baselineSince = new Date(now - baselineDays * 24 * 60 * 60 * 1000);

	// Get recent predictions
	const recentPredictions = await db.query.patternPrediction.findMany({
		where: (p, { gte }) => gte(p.createdAt, recentSince),
	});

	// Get baseline predictions (excluding recent)
	const baselinePredictions = await db.query.patternPrediction.findMany({
		where: (p, { and, gte, lt }) =>
			and(gte(p.createdAt, baselineSince), lt(p.createdAt, recentSince)),
	});

	const toPrediction = (p: typeof recentPredictions[0]) => ({
		id: p.id,
		predictedConfidence: p.predictedConfidence,
		wasCorrect: p.wasCorrect,
		validatedAt: p.validatedAt,
	});

	const recentMetrics = computeCalibrationMetrics(
		recentPredictions.map(toPrediction),
	);
	const baselineMetrics = computeCalibrationMetrics(
		baselinePredictions.map(toPrediction),
	);

	const alert = detectDrift(recentMetrics, baselineMetrics);

	return c.json({
		hasDrift: alert !== null,
		alert,
		recent: recentMetrics,
		baseline: baselineMetrics,
	});
});

// @ts-expect-error Hono OpenAPI type limitation with union response types
calibrationApp.openapi(validatePredictionRoute, async (c) => {
	const { id } = c.req.valid('param');
	const body = c.req.valid('json');

	const prediction = await db.query.patternPrediction.findFirst({
		where: (p, { eq }) => eq(p.id, id),
	});

	if (!prediction) {
		return c.json(
			{ error: { code: 'NOT_FOUND', message: 'Prediction not found' } },
			404,
		);
	}

	await db
		.update(patternPrediction)
		.set({
			wasCorrect: body.wasCorrect,
			validatedAt: new Date(),
			validationSource: body.validationSource,
		})
		.where(eq(patternPrediction.id, id));

	return c.json({ success: true });
});

calibrationApp.openapi(recalibrateRoute, async (c) => {
	const body = c.req.valid('json');

	const since = new Date(Date.now() - body.days * 24 * 60 * 60 * 1000);

	const predictions = await db.query.patternPrediction.findMany({
		where: (p, { and, gte, isNotNull }) =>
			and(gte(p.createdAt, since), isNotNull(p.wasCorrect)),
	});

	const validatedPredictions = predictions
		.filter((p) => p.wasCorrect !== null)
		.map((p) => ({
			id: p.id,
			predictedConfidence: p.predictedConfidence,
			wasCorrect: p.wasCorrect,
			validatedAt: p.validatedAt,
		}));

	if (body.method === 'platt') {
		const params = fitPlattScaling(validatedPredictions);
		return c.json({
			method: 'platt',
			samplesUsed: validatedPredictions.length,
			parameters: params,
		});
	}

	const mapping = fitIsotonicRegression(validatedPredictions);
	return c.json({
		method: 'isotonic',
		samplesUsed: validatedPredictions.length,
		parameters: mapping,
	});
});

export { calibrationApp };
