import { createRoute, OpenAPIHono, z } from '@hono/zod-openapi';
import { eq, sql } from 'drizzle-orm';
import { ulid } from 'ulid';

import {
	crashPattern,
	patternModCorrelation,
	db,
} from '@/db/index';
import {
	extractModNames,
	computeModCorrelation,
	isSignificant,
} from '@/lib/mod-correlation';
import { findMatchingPriors, applyBayesianUpdate } from '@/lib/priors';

// Schemas
const PatternSchema = z
	.object({
		id: z.string(),
		gameId: z.string(),
		crashHash: z.string(),
		patternName: z.string().nullable(),
		occurrenceCount: z.number(),
		firstSeenAt: z.number(),
		lastSeenAt: z.number(),
		knownFix: z.string().nullable(),
		isResolved: z.boolean(),
	})
	.openapi('Pattern');

const ModCorrelationSchema = z
	.object({
		modName: z.string(),
		modFingerprint: z.string().nullable(),
		crashesWithMod: z.number(),
		crashesWithoutMod: z.number(),
		totalCrashes: z.number(),
		lift: z.number(),
		correlationScore: z.number(),
		confidence: z.number(),
		isSignificant: z.boolean(),
	})
	.openapi('ModCorrelation');

const PriorMatchSchema = z
	.object({
		priorId: z.string(),
		patternName: z.string(),
		knownFix: z.string().nullable(),
		suspectedMods: z.array(z.string()),
		source: z.string(),
		matchScore: z.number(),
		posteriorConfidence: z.number(),
		dataPoints: z.number(),
	})
	.openapi('PriorMatch');

const PatternDetailSchema = z
	.object({
		pattern: PatternSchema,
		correlations: z.array(ModCorrelationSchema),
		priorMatches: z.array(PriorMatchSchema),
	})
	.openapi('PatternDetail');

const ErrorSchema = z
	.object({
		error: z.object({
			code: z.string(),
			message: z.string(),
		}),
	})
	.openapi('Error');

// Routes
const listPatternsRoute = createRoute({
	method: 'get',
	path: '/',
	tags: ['Patterns'],
	summary: 'List crash patterns',
	description: 'Get crash patterns for a game, ordered by occurrence count',
	request: {
		query: z.object({
			gameId: z.string().openapi({ description: 'Filter by game' }),
			limit: z.coerce.number().min(1).max(100).default(20),
			offset: z.coerce.number().min(0).default(0),
		}),
	},
	responses: {
		200: {
			content: {
				'application/json': {
					schema: z.object({
						patterns: z.array(PatternSchema),
						total: z.number(),
					}),
				},
			},
			description: 'List of patterns',
		},
	},
});

const getPatternRoute = createRoute({
	method: 'get',
	path: '/{id}',
	tags: ['Patterns'],
	summary: 'Get pattern details',
	description: 'Get detailed pattern information including mod correlations',
	request: {
		params: z.object({
			id: z.string(),
		}),
	},
	responses: {
		200: {
			content: {
				'application/json': {
					schema: PatternDetailSchema,
				},
			},
			description: 'Pattern details',
		},
		404: {
			content: {
				'application/json': {
					schema: ErrorSchema,
				},
			},
			description: 'Pattern not found',
		},
	},
});

const computeCorrelationsRoute = createRoute({
	method: 'post',
	path: '/{id}/correlations',
	tags: ['Patterns'],
	summary: 'Compute mod correlations',
	description: 'Recompute mod correlation scores for a pattern',
	request: {
		params: z.object({
			id: z.string(),
		}),
	},
	responses: {
		200: {
			content: {
				'application/json': {
					schema: z.object({
						computed: z.number(),
						significant: z.number(),
					}),
				},
			},
			description: 'Correlation computation complete',
		},
		404: {
			content: {
				'application/json': {
					schema: ErrorSchema,
				},
			},
			description: 'Pattern not found',
		},
	},
});

// App and handlers
const patternsApp = new OpenAPIHono();

patternsApp.openapi(listPatternsRoute, async (c) => {
	const { gameId, limit, offset } = c.req.valid('query');

	const patterns = await db.query.crashPattern.findMany({
		where: (p, { eq }) => eq(p.gameId, gameId),
		orderBy: (p, { desc }) => [desc(p.occurrenceCount)],
		limit,
		offset,
	});

	// Get total count
	const countResult = await db
		.select({ count: sql<number>`count(*)` })
		.from(crashPattern)
		.where(eq(crashPattern.gameId, gameId));

	const total = countResult[0]?.count ?? 0;

	return c.json({
		patterns: patterns.map((p) => ({
			id: p.id,
			gameId: p.gameId,
			crashHash: p.crashHash,
			patternName: p.patternName,
			occurrenceCount: p.occurrenceCount,
			firstSeenAt: p.firstSeenAt.getTime(),
			lastSeenAt: p.lastSeenAt.getTime(),
			knownFix: p.knownFix,
			isResolved: p.isResolved,
		})),
		total,
	});
});

// @ts-expect-error Hono OpenAPI type limitation with union response types
patternsApp.openapi(getPatternRoute, async (c) => {
	const { id } = c.req.valid('param');

	const pattern = await db.query.crashPattern.findFirst({
		where: (p, { eq }) => eq(p.id, id),
	});

	if (!pattern) {
		return c.json(
			{ error: { code: 'NOT_FOUND', message: 'Pattern not found' } },
			404,
		);
	}

	// Get correlations
	const correlations = await db.query.patternModCorrelation.findMany({
		where: (pc, { eq }) => eq(pc.patternId, id),
		orderBy: (pc, { desc }) => [desc(pc.correlationScore)],
		limit: 20,
	});

	// Get prior matches
	const priors = await db.query.crashPrior.findMany({
		where: (p, { eq }) => eq(p.gameId, pattern.gameId),
	});

	// Get a sample crash to match priors against
	const sampleCrash = await db.query.crashReport.findFirst({
		where: (r, { eq }) => eq(r.crashHash, pattern.crashHash),
	});

	let priorMatches: Array<{
		priorId: string;
		patternName: string;
		knownFix: string | null;
		suspectedMods: string[];
		source: string;
		matchScore: number;
		posteriorConfidence: number;
		dataPoints: number;
	}> = [];

	if (sampleCrash && priors.length > 0) {
		const priorDtos = priors.map((p) => ({
			id: p.id,
			gameId: p.gameId,
			signaturePattern: p.signaturePattern,
			matchType: p.matchType as 'exact' | 'contains' | 'regex',
			patternName: p.patternName,
			knownFix: p.knownFix,
			suspectedMods: p.suspectedModsJson
				? JSON.parse(p.suspectedModsJson)
				: [],
			source: p.source,
			sourceUrl: p.sourceUrl,
			priorConfidence: p.priorConfidence,
		}));

		const matches = findMatchingPriors(
			sampleCrash.stackTrace,
			sampleCrash.faultingModule,
			pattern.gameId,
			priorDtos,
		);

		// Apply Bayesian update with pattern occurrence count
		priorMatches = matches.slice(0, 5).map((m) => {
			const updated = applyBayesianUpdate(
				m,
				pattern.occurrenceCount,
				Math.min(95, 50 + pattern.occurrenceCount * 2), // Simple confidence from count
			);
			return {
				priorId: m.prior.id,
				patternName: m.prior.patternName,
				knownFix: m.prior.knownFix,
				suspectedMods: m.prior.suspectedMods,
				source: m.prior.source,
				matchScore: updated.matchScore,
				posteriorConfidence: updated.posteriorConfidence,
				dataPoints: updated.dataPoints,
			};
		});
	}

	return c.json({
		pattern: {
			id: pattern.id,
			gameId: pattern.gameId,
			crashHash: pattern.crashHash,
			patternName: pattern.patternName,
			occurrenceCount: pattern.occurrenceCount,
			firstSeenAt: pattern.firstSeenAt.getTime(),
			lastSeenAt: pattern.lastSeenAt.getTime(),
			knownFix: pattern.knownFix,
			isResolved: pattern.isResolved,
		},
		correlations: correlations.map((c) => ({
			modName: c.modName,
			modFingerprint: c.modFingerprint,
			crashesWithMod: c.crashesWithMod,
			crashesWithoutMod: c.crashesWithoutMod,
			totalCrashes: c.totalCrashes,
			lift: c.lift / 100, // Stored as percentage
			correlationScore: c.correlationScore,
			confidence: c.confidence,
			isSignificant: c.correlationScore >= 60 && c.confidence >= 50,
		})),
		priorMatches,
	});
});

// @ts-expect-error Hono OpenAPI type limitation with union response types
patternsApp.openapi(computeCorrelationsRoute, async (c) => {
	const { id } = c.req.valid('param');

	const pattern = await db.query.crashPattern.findFirst({
		where: (p, { eq }) => eq(p.id, id),
	});

	if (!pattern) {
		return c.json(
			{ error: { code: 'NOT_FOUND', message: 'Pattern not found' } },
			404,
		);
	}

	// Get all crashes for this pattern
	const crashes = await db.query.crashReport.findMany({
		where: (r, { eq }) => eq(r.crashHash, pattern.crashHash),
	});

	if (crashes.length === 0) {
		return c.json({ computed: 0, significant: 0 });
	}

	// Count mod occurrences
	const modCounts = new Map<string, { with: number; fingerprint?: string }>();

	for (const crash of crashes) {
		const mods = extractModNames(crash.loadOrderJson);
		const seenMods = new Set<string>();

		for (const mod of mods) {
			if (!seenMods.has(mod.modName)) {
				seenMods.add(mod.modName);
				const existing = modCounts.get(mod.modName) || { with: 0 };
				existing.with++;
				if (mod.modFingerprint) {
					existing.fingerprint = mod.modFingerprint;
				}
				modCounts.set(mod.modName, existing);
			}
		}
	}

	const now = new Date();
	const totalCrashes = crashes.length;
	let computed = 0;
	let significant = 0;

	// Delete existing correlations for this pattern
	await db
		.delete(patternModCorrelation)
		.where(eq(patternModCorrelation.patternId, id));

	// Compute and store correlations
	for (const [modName, counts] of modCounts) {
		// Estimate base rate (how common is this mod across all crashes for the game)
		// For now, use a simple heuristic based on occurrence in this pattern
		const baseRate = Math.max(0.05, counts.with / totalCrashes);

		const result = computeModCorrelation(
			modName,
			counts.with,
			totalCrashes - counts.with,
			baseRate,
			counts.fingerprint,
		);

		await db.insert(patternModCorrelation).values({
			id: ulid(),
			patternId: id,
			gameId: pattern.gameId,
			modName,
			modFingerprint: counts.fingerprint || null,
			crashesWithMod: result.crashesWithMod,
			crashesWithoutMod: result.crashesWithoutMod,
			totalCrashes: result.totalCrashes,
			lift: Math.round(result.lift * 100),
			correlationScore: result.correlationScore,
			confidence: result.confidence,
			updatedAt: now,
		});

		computed++;
		if (isSignificant(result)) {
			significant++;
		}
	}

	return c.json({ computed, significant });
});

export { patternsApp };
