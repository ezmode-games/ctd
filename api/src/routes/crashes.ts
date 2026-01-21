import { createRoute, OpenAPIHono, z } from '@hono/zod-openapi';
import { eq } from 'drizzle-orm';
import Sqids from 'sqids';
import { ulid } from 'ulid';

import { crashPattern, crashReport, db } from '@/db/index';
import { computeCrashHash } from '@/lib/crash-hash';

// Schemas
const CreateCrashReportSchema = z
	.object({
		schemaVersion: z.number().int().min(1).default(1),
		gameId: z.string().min(1).openapi({ example: 'skyrim' }),
		stackTrace: z.string().min(1).max(100000).openapi({
			description: 'Stack trace from the crash',
		}),
		crashHash: z.string().min(1).max(64).optional().openapi({
			description: 'Pre-computed crash hash (optional)',
		}),
		exceptionCode: z
			.string()
			.max(50)
			.optional()
			.openapi({ example: '0xC0000005' }),
		exceptionAddress: z.string().max(50).optional(),
		faultingModule: z
			.string()
			.max(255)
			.optional()
			.openapi({ example: 'SkyrimSE.exe' }),
		gameVersion: z.string().min(1).max(50).openapi({ example: '1.6.1170' }),
		scriptExtenderVersion: z
			.string()
			.max(50)
			.optional()
			.openapi({ example: '2.2.3' }),
		osVersion: z
			.string()
			.max(100)
			.optional()
			.openapi({ example: 'Windows 10.0.19045' }),
		loadOrderJson: z
			.string()
			.refine(
				(val) => {
					try {
						const parsed = JSON.parse(val);
						return Array.isArray(parsed);
					} catch {
						return false;
					}
				},
				{ message: 'loadOrderJson must be a valid JSON array' },
			)
			.openapi({
				description: 'JSON array of load order',
				example: '[{"name":"Skyrim.esm","enabled":true,"index":0}]',
			}),
		pluginCount: z.number().int().min(0).max(10000),
		crashedAt: z.number().int().positive().openapi({
			description: 'Timestamp when crash occurred (ms since epoch)',
		}),
		notes: z.string().max(5000).optional(),
	})
	.openapi('CreateCrashReport');

const CrashReportCreatedSchema = z
	.object({
		id: z.string().openapi({ description: 'Crash report ID' }),
		shareToken: z
			.string()
			.openapi({ description: 'Token for sharing the report' }),
	})
	.openapi('CrashReportCreated');

const CrashPatternSchema = z
	.object({
		id: z.string(),
		patternName: z.string().nullable(),
		occurrenceCount: z.number(),
		knownFix: z.string().nullable(),
	})
	.openapi('CrashPattern');

const CrashReportSchema = z
	.object({
		id: z.string(),
		schemaVersion: z.number(),
		gameId: z.string(),
		crashHash: z.string(),
		stackTrace: z.string(),
		exceptionCode: z.string().nullable(),
		exceptionAddress: z.string().nullable(),
		faultingModule: z.string().nullable(),
		gameVersion: z.string(),
		scriptExtenderVersion: z.string().nullable(),
		osVersion: z.string().nullable(),
		loadOrder: z.array(z.object({})),
		pluginCount: z.number(),
		crashedAt: z.number(),
		submittedAt: z.number(),
		isPublic: z.boolean(),
		notes: z.string().nullable(),
		pattern: CrashPatternSchema.nullable(),
	})
	.openapi('CrashReport');

const ValidationErrorSchema = z
	.object({
		error: z.object({
			code: z.string().openapi({ example: 'VALIDATION_ERROR' }),
			issues: z.array(z.object({})),
		}),
	})
	.openapi('ValidationError');

const ErrorSchema = z
	.object({
		error: z.object({
			code: z.string(),
			message: z.string(),
		}),
	})
	.openapi('Error');

// Routes
const submitCrashRoute = createRoute({
	method: 'post',
	path: '/',
	tags: ['Crashes'],
	summary: 'Submit crash report',
	description: 'Submit a new crash report from a game client',
	security: [{ apiKey: [] }],
	request: {
		body: {
			content: {
				'application/json': {
					schema: CreateCrashReportSchema,
				},
			},
			required: true,
		},
	},
	responses: {
		201: {
			content: {
				'application/json': {
					schema: CrashReportCreatedSchema,
				},
			},
			description: 'Crash report submitted successfully',
		},
		422: {
			content: {
				'application/json': {
					schema: ValidationErrorSchema,
				},
			},
			description: 'Validation error',
		},
	},
});

const getCrashRoute = createRoute({
	method: 'get',
	path: '/{id}',
	tags: ['Crashes'],
	summary: 'Get crash report',
	description:
		'Retrieve a crash report by ID. Requires share token for non-public reports.',
	request: {
		params: z.object({
			id: z.string().openapi({ description: 'Crash report ID (ULID)' }),
		}),
		query: z.object({
			token: z.string().optional().openapi({
				description: 'Share token for accessing non-public reports',
			}),
		}),
	},
	responses: {
		200: {
			content: {
				'application/json': {
					schema: CrashReportSchema,
				},
			},
			description: 'Crash report details',
		},
		404: {
			content: {
				'application/json': {
					schema: ErrorSchema,
				},
			},
			description: 'Crash report not found',
		},
	},
});

// App and handlers
const crashesApp = new OpenAPIHono();

// Sqids for URL-safe share tokens
const sqids = new Sqids({
	minLength: 12,
});

function generateShareToken(): string {
	// Use crypto random values for unpredictable tokens
	const randomValues = [
		Math.floor(Math.random() * Number.MAX_SAFE_INTEGER),
		Math.floor(Math.random() * Number.MAX_SAFE_INTEGER),
	];
	return sqids.encode(randomValues);
}

crashesApp.openapi(submitCrashRoute, async (c) => {
	const body = c.req.valid('json');

	const id = ulid();
	const shareToken = generateShareToken();
	const crashHash = body.crashHash || computeCrashHash(body.stackTrace);
	const now = new Date();

	await db.insert(crashReport).values({
		id,
		schemaVersion: body.schemaVersion,
		gameId: body.gameId,
		crashHash,
		stackTrace: body.stackTrace,
		exceptionCode: body.exceptionCode,
		exceptionAddress: body.exceptionAddress,
		faultingModule: body.faultingModule,
		gameVersion: body.gameVersion,
		scriptExtenderVersion: body.scriptExtenderVersion,
		osVersion: body.osVersion,
		loadOrderJson: body.loadOrderJson,
		pluginCount: body.pluginCount,
		crashedAt: new Date(body.crashedAt),
		submittedAt: now,
		shareToken,
		notes: body.notes,
		createdAt: now,
	});

	// Update or create crash pattern
	const existingPattern = await db.query.crashPattern.findFirst({
		where: (pattern, { eq }) => eq(pattern.crashHash, crashHash),
	});

	if (existingPattern) {
		await db
			.update(crashPattern)
			.set({
				occurrenceCount: existingPattern.occurrenceCount + 1,
				lastSeenAt: now,
				updatedAt: now,
			})
			.where(eq(crashPattern.id, existingPattern.id));
	} else {
		await db.insert(crashPattern).values({
			id: ulid(),
			gameId: body.gameId,
			crashHash,
			occurrenceCount: 1,
			firstSeenAt: now,
			lastSeenAt: now,
			createdAt: now,
			updatedAt: now,
		});
	}

	return c.json({ id, shareToken }, 201);
});

crashesApp.openapi(getCrashRoute, async (c) => {
	const { id } = c.req.valid('param');
	const { token } = c.req.valid('query');

	const report = await db.query.crashReport.findFirst({
		where: (r, { eq }) => eq(r.id, id),
	});

	if (!report) {
		return c.json(
			{ error: { code: 'NOT_FOUND', message: 'Crash report not found' } },
			404,
		);
	}

	// Check access: public or valid share token
	if (!report.isPublic && report.shareToken !== token) {
		return c.json(
			{ error: { code: 'NOT_FOUND', message: 'Crash report not found' } },
			404,
		);
	}

	// Get pattern info if exists
	const pattern = await db.query.crashPattern.findFirst({
		where: (p, { eq }) => eq(p.crashHash, report.crashHash),
	});

	return c.json({
		id: report.id,
		schemaVersion: report.schemaVersion,
		gameId: report.gameId,
		crashHash: report.crashHash,
		stackTrace: report.stackTrace,
		exceptionCode: report.exceptionCode,
		exceptionAddress: report.exceptionAddress,
		faultingModule: report.faultingModule,
		gameVersion: report.gameVersion,
		scriptExtenderVersion: report.scriptExtenderVersion,
		osVersion: report.osVersion,
		loadOrder: JSON.parse(report.loadOrderJson),
		pluginCount: report.pluginCount,
		crashedAt: report.crashedAt.getTime(),
		submittedAt: report.submittedAt.getTime(),
		isPublic: report.isPublic,
		notes: report.notes,
		pattern: pattern
			? {
					id: pattern.id,
					patternName: pattern.patternName,
					occurrenceCount: pattern.occurrenceCount,
					knownFix: pattern.knownFix,
				}
			: null,
	});
});

export { crashesApp };
