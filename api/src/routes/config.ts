import { createRoute, OpenAPIHono, z } from '@hono/zod-openapi';

import { db } from '@/db/index';
import { hashApiKey, isValidApiKeyFormat } from '@/lib/api-key';

// Schemas
const ConfigQuerySchema = z.object({
	key: z.string().openapi({
		description: 'Your API key',
		example: 'ctd_a8Kj2mNp4qRs6tUv8wXy0zB3dF5gH7jL',
	}),
	format: z
		.enum(['toml', 'json'])
		.default('toml')
		.openapi({ description: 'Output format' }),
});

const ConfigJsonSchema = z
	.object({
		server_url: z.string(),
		api_key: z.string(),
	})
	.openapi('ConfigJson');

const ErrorSchema = z
	.object({
		error: z.object({
			code: z.string(),
			message: z.string(),
		}),
	})
	.openapi('Error');

// Routes
const getConfigRoute = createRoute({
	method: 'get',
	path: '/',
	tags: ['Config'],
	summary: 'Download config file',
	description:
		'Download a pre-configured ctd.toml file with your API key embedded. This file should be placed in your game directory.',
	request: {
		query: ConfigQuerySchema,
	},
	responses: {
		200: {
			content: {
				'application/toml': {
					schema: z
						.string()
						.openapi({
							example:
								'[api]\nurl = "https://your-server.example.com"\napi_key = "ctd_..."',
						}),
				},
				'application/json': {
					schema: ConfigJsonSchema,
				},
			},
			description: 'Configuration file',
		},
		400: {
			content: {
				'application/json': {
					schema: ErrorSchema,
				},
			},
			description: 'Bad request',
		},
		401: {
			content: {
				'application/json': {
					schema: ErrorSchema,
				},
			},
			description: 'Invalid or expired API key',
		},
	},
});

/**
 * Escape a string for TOML basic string format.
 * Escapes backslashes, quotes, and control characters.
 */
function escapeTomlString(str: string): string {
	return str
		.replace(/\\/g, '\\\\')
		.replace(/"/g, '\\"')
		.replace(/\n/g, '\\n')
		.replace(/\r/g, '\\r')
		.replace(/\t/g, '\\t');
}

// App and handlers
const configApp = new OpenAPIHono();

configApp.openapi(getConfigRoute, async (c) => {
	const { key, format } = c.req.valid('query');

	// Validate key format
	if (!isValidApiKeyFormat(key)) {
		return c.json(
			{ error: { code: 'UNAUTHORIZED', message: 'Invalid API key' } },
			401,
		);
	}

	// Validate key exists in database
	const keyHash = hashApiKey(key);
	const existingKey = await db.query.apiKey.findFirst({
		where: (k, { eq }) => eq(k.keyHash, keyHash),
	});

	if (!existingKey) {
		return c.json(
			{ error: { code: 'UNAUTHORIZED', message: 'Invalid API key' } },
			401,
		);
	}

	// Check if key has expired
	if (existingKey.expiresAt && existingKey.expiresAt <= new Date()) {
		return c.json(
			{ error: { code: 'UNAUTHORIZED', message: 'API key has expired' } },
			401,
		);
	}

	// Get server URL from environment or derive from request headers (with validation)
	let serverUrl = process.env.CTD_SERVER_URL;

	if (!serverUrl) {
		const forwardedProto = c.req.header('x-forwarded-proto');
		const hostHeader = c.req.header('host');

		// Validate headers to prevent injection
		const safeProto =
			forwardedProto === 'http' || forwardedProto === 'https'
				? forwardedProto
				: 'http';
		const safeHost =
			hostHeader && /^[a-zA-Z0-9.-]+(:\d+)?$/.test(hostHeader)
				? hostHeader
				: 'localhost:3000';

		serverUrl = `${safeProto}://${safeHost}`;
	}

	// Return based on format
	if (format === 'json') {
		return c.json({
			server_url: serverUrl,
			api_key: key,
		});
	}

	// TOML format with proper escaping
	const tomlContent = `# CTD - Crash to Desktop Reporter
# https://ctd.ezmode.games
#
# Generated for OSS server

[api]
url = "${escapeTomlString(serverUrl)}"
api_key = "${escapeTomlString(key)}"
`;

	return c.body(tomlContent, 200, {
		'Content-Type': 'application/toml',
		'Content-Disposition': 'attachment; filename="ctd.toml"',
	});
});

export { configApp };
