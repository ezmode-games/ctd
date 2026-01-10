import { swaggerUI } from '@hono/swagger-ui';
import { readFileSync } from 'node:fs';
import { Hono } from 'hono';

const docs = new Hono();

// Load OpenAPI spec at startup
const openapiSpec = readFileSync(
	new URL('../openapi.yaml', import.meta.url),
	'utf-8',
);

// GET /docs/openapi.yaml - Raw OpenAPI spec
docs.get('/openapi.yaml', (c) => {
	return c.body(openapiSpec, 200, {
		'Content-Type': 'application/yaml',
	});
});

// GET /docs - Swagger UI using @hono/swagger-ui
docs.get('/', swaggerUI({ url: '/docs/openapi.yaml' }));

export { docs };
