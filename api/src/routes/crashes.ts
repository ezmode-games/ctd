import { zValidator } from '@hono/zod-validator';
import { eq } from 'drizzle-orm';
import { Hono } from 'hono';
import Sqids from 'sqids';
import { ulid } from 'ulid';

import {
	crashPattern,
	crashReport,
	createCrashReportSchema,
	db,
} from '@/db/index';
import { computeCrashHash } from '@/lib/crash-hash';

const crashes = new Hono();

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

// POST /crashes - Submit a crash report
crashes.post(
	'/',
	zValidator('json', createCrashReportSchema, (result, c) => {
		if (!result.success) {
			return c.json(
				{ error: { code: 'VALIDATION_ERROR', issues: result.error.issues } },
				422,
			);
		}
	}),
	async (c) => {
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
	},
);

// GET /crashes/:id - Get a crash report
crashes.get('/:id', async (c) => {
	const id = c.req.param('id');
	const token = c.req.query('token');

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

export { crashes };
