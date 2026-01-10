import { Hono } from 'hono';

import { db } from '@/db/index';
import { hashApiKey, isValidApiKeyFormat } from '@/lib/api-key';

const config = new Hono();

// GET /config - Download configuration file
// Query params:
//   key: API key (required)
//   format: 'toml' | 'json' (default: 'toml')
config.get('/', async (c) => {
	const key = c.req.query('key');
	const format = c.req.query('format') || 'toml';

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

	// Get server URL from environment or request origin
	const serverUrl =
		process.env.CTD_SERVER_URL ||
		`${c.req.header('x-forwarded-proto') || 'http'}://${c.req.header('host')}`;

	// Return based on format
	if (format === 'json') {
		return c.json({
			server_url: serverUrl,
			api_key: key,
		});
	}

	// Default: TOML format
	const tomlContent = `# CTD - Crash to Desktop Reporter
# https://ctd.ezmode.games
#
# Generated for OSS server

[api]
url = "${serverUrl}"
api_key = "${key}"
`;

	return c.body(tomlContent, 200, {
		'Content-Type': 'application/toml',
		'Content-Disposition': 'attachment; filename="ctd.toml"',
	});
});

export { config };
