import { zValidator } from '@hono/zod-validator';
import { eq } from 'drizzle-orm';
import { Hono } from 'hono';
import { ulid } from 'ulid';
import { z } from 'zod';

import { apiKey, db } from '@/db/index';
import {
	generateApiKey,
	getKeyPrefix,
	hashApiKey,
} from '@/lib/api-key';

const apiKeys = new Hono();

// Request schemas
const createApiKeySchema = z.object({
	name: z.string().min(1).max(100).optional(),
	expiresIn: z.number().int().positive().optional(), // milliseconds
});

// POST /api-keys - Generate new API key
apiKeys.post(
	'/',
	zValidator('json', createApiKeySchema, (result, c) => {
		if (!result.success) {
			return c.json(
				{ error: { code: 'VALIDATION_ERROR', issues: result.error.issues } },
				400,
			);
		}
		return undefined;
	}),
	async (c) => {
		const body = c.req.valid('json');

		const id = ulid();
		const key = generateApiKey();
		const keyHash = hashApiKey(key);
		const keyPrefix = getKeyPrefix(key);
		const now = new Date();

		const expiresAt = body.expiresIn
			? new Date(now.getTime() + body.expiresIn)
			: null;

		await db.insert(apiKey).values({
			id,
			name: body.name || 'default',
			keyHash,
			keyPrefix,
			expiresAt,
			createdAt: now,
		});

		return c.json(
			{
				key, // Only returned on creation
				id,
				name: body.name || 'default',
				keyPrefix,
				expiresAt: expiresAt?.getTime() ?? null,
				createdAt: now.getTime(),
			},
			201,
		);
	},
);

// GET /api-keys - List keys (shows prefix only)
apiKeys.get('/', async (c) => {
	const keys = await db.query.apiKey.findMany({
		orderBy: (k, { desc }) => [desc(k.createdAt)],
	});

	return c.json({
		keys: keys.map((k) => ({
			id: k.id,
			name: k.name,
			keyPrefix: k.keyPrefix,
			lastUsedAt: k.lastUsedAt?.getTime() ?? null,
			expiresAt: k.expiresAt?.getTime() ?? null,
			createdAt: k.createdAt.getTime(),
		})),
	});
});

// DELETE /api-keys/:id - Revoke key
apiKeys.delete('/:id', async (c) => {
	const id = c.req.param('id');

	const existing = await db.query.apiKey.findFirst({
		where: (k, { eq }) => eq(k.id, id),
	});

	if (!existing) {
		return c.json(
			{ error: { code: 'NOT_FOUND', message: 'API key not found' } },
			404,
		);
	}

	await db.delete(apiKey).where(eq(apiKey.id, id));

	return c.json({ success: true });
});

export { apiKeys };
