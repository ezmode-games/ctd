import { eq } from 'drizzle-orm';
import * as HttpStatusCodes from 'stoker/http-status-codes';
import { uuidv7 } from 'uuidv7';

import { crashPattern, crashReport, db } from '@/db/index';
import type { AppRouteHandler } from '@/lib/types';

import type { SubmitCrashReportRoute } from './crash-reports.routes';

function generateShareToken(): string {
	const chars =
		'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
	let token = '';
	for (let i = 0; i < 16; i++) {
		token += chars.charAt(Math.floor(Math.random() * chars.length));
	}
	return token;
}

function computeCrashHash(stackTrace: string): string {
	// Simple hash for now - will be replaced with proper implementation in Issue #5
	const normalized = stackTrace
		.split('\n')
		.slice(0, 10)
		.map((line) => line.trim().toLowerCase())
		.join('|');

	// Simple string hash
	let hash = 0;
	for (let i = 0; i < normalized.length; i++) {
		const char = normalized.charCodeAt(i);
		hash = (hash << 5) - hash + char;
		hash = hash & hash;
	}
	return Math.abs(hash).toString(16).padStart(16, '0').slice(0, 16);
}

export const submitCrashReport: AppRouteHandler<
	SubmitCrashReportRoute
> = async (c) => {
	const body = c.req.valid('json');

	const id = uuidv7();
	const shareToken = generateShareToken();
	const crashHash = body.crashHash || computeCrashHash(body.stackTrace);
	const now = new Date();

	// Insert crash report
	await db.insert(crashReport).values({
		id,
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
			id: uuidv7(),
			gameId: body.gameId,
			crashHash,
			occurrenceCount: 1,
			firstSeenAt: now,
			lastSeenAt: now,
			createdAt: now,
			updatedAt: now,
		});
	}

	return c.json(
		{
			id,
			shareToken,
		},
		HttpStatusCodes.CREATED,
	);
};
