import { createRoute, OpenAPIHono, z } from '@hono/zod-openapi';
import { eq } from 'drizzle-orm';
import { ulid } from 'ulid';

import { apiKey, db } from '@/db/index';
import { generateApiKey, getKeyPrefix, hashApiKey } from '@/lib/api-key';

// Schemas
const ApiKeyCreatedSchema = z
	.object({
		key: z
			.string()
			.openapi({ example: 'ctd_a8Kj2mNp4qRs6tUv8wXy0zB3dF5gH7jL' }),
		id: z.string(),
		name: z.string(),
		keyPrefix: z.string().openapi({ example: 'ctd_a8Kj' }),
		expiresAt: z.number().nullable(),
		createdAt: z.number(),
	})
	.openapi('ApiKeyCreated');

const ApiKeyInfoSchema = z
	.object({
		id: z.string(),
		name: z.string(),
		keyPrefix: z.string().openapi({ example: 'ctd_a8Kj' }),
		lastUsedAt: z.number().nullable(),
		expiresAt: z.number().nullable(),
		createdAt: z.number(),
	})
	.openapi('ApiKeyInfo');

const ApiKeyListSchema = z
	.object({
		keys: z.array(ApiKeyInfoSchema),
	})
	.openapi('ApiKeyList');

const CreateApiKeyBodySchema = z
	.object({
		name: z
			.string()
			.min(1)
			.max(100)
			.optional()
			.openapi({ example: 'production' }),
		expiresIn: z.number().int().positive().optional().openapi({
			description: 'Expiration time in milliseconds',
			example: 86400000,
		}),
	})
	.openapi('CreateApiKeyBody');

const ErrorSchema = z
	.object({
		error: z.object({
			code: z.string(),
			message: z.string(),
		}),
	})
	.openapi('Error');

// Routes
const createApiKeyRoute = createRoute({
	method: 'post',
	path: '/',
	tags: ['API Keys'],
	summary: 'Generate new API key',
	description: 'Create a new API key. The full key is only returned once.',
	request: {
		body: {
			content: {
				'application/json': {
					schema: CreateApiKeyBodySchema,
				},
			},
		},
	},
	responses: {
		201: {
			content: {
				'application/json': {
					schema: ApiKeyCreatedSchema,
				},
			},
			description: 'API key created successfully',
		},
		400: {
			content: {
				'application/json': {
					schema: ErrorSchema,
				},
			},
			description: 'Validation error',
		},
	},
});

const listApiKeysRoute = createRoute({
	method: 'get',
	path: '/',
	tags: ['API Keys'],
	summary: 'List API keys',
	description: 'List all API keys. Only the key prefix is shown for security.',
	responses: {
		200: {
			content: {
				'application/json': {
					schema: ApiKeyListSchema,
				},
			},
			description: 'List of API keys',
		},
	},
});

const deleteApiKeyRoute = createRoute({
	method: 'delete',
	path: '/{id}',
	tags: ['API Keys'],
	summary: 'Revoke API key',
	description: 'Permanently delete an API key',
	request: {
		params: z.object({
			id: z.string().openapi({ description: 'API key ID (ULID)' }),
		}),
	},
	responses: {
		200: {
			content: {
				'application/json': {
					schema: z.object({ success: z.boolean() }),
				},
			},
			description: 'Key revoked successfully',
		},
		404: {
			content: {
				'application/json': {
					schema: ErrorSchema,
				},
			},
			description: 'API key not found',
		},
	},
});

// App and handlers
const apiKeysApp = new OpenAPIHono();

apiKeysApp.openapi(createApiKeyRoute, async (c) => {
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
			key,
			id,
			name: body.name || 'default',
			keyPrefix,
			expiresAt: expiresAt?.getTime() ?? null,
			createdAt: now.getTime(),
		},
		201,
	);
});

apiKeysApp.openapi(listApiKeysRoute, async (c) => {
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

apiKeysApp.openapi(deleteApiKeyRoute, async (c) => {
	const { id } = c.req.valid('param');

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

export { apiKeysApp };
