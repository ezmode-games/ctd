import { Hono } from 'hono';

import { db } from '@/db/index';
import { hashApiKey, isValidApiKeyFormat } from '@/lib/api-key';

const config = new Hono();

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

// GET /config - Download configuration file
// Query params:
//   key: API key (required)
//   format: 'toml' | 'json' (default: 'toml')
config.get('/', async (c) => {
	const key = c.req.query('key');
	const rawFormat = c.req.query('format');

	// Validate format param
	let format: 'toml' | 'json';
	if (!rawFormat) {
		format = 'toml';
	} else if (rawFormat === 'toml' || rawFormat === 'json') {
		format = rawFormat;
	} else {
		return c.json(
			{ error: { code: 'BAD_REQUEST', message: 'Invalid format parameter. Use "toml" or "json"' } },
			400,
		);
	}

	// Validate key param exists
	if (!key) {
		return c.json(
			{ error: { code: 'BAD_REQUEST', message: 'Missing key parameter' } },
			400,
		);
	}

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

export { config };
