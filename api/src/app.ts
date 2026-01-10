import { swaggerUI } from '@hono/swagger-ui';
import { OpenAPIHono } from '@hono/zod-openapi';
import { cors } from 'hono/cors';
import { requestId } from 'hono/request-id';

import { apiKeysApp } from '@/routes/api-keys';
import { configApp } from '@/routes/config';
import { crashesApp } from '@/routes/crashes';

const app = new OpenAPIHono();

// Middleware
app.use('*', cors());
app.use('*', requestId());

// Health check
app.get('/health', (c) => {
	return c.json({ status: 'ok', version: '0.1.0' });
});

// Mount route apps
app.route('/api-keys', apiKeysApp);
app.route('/config', configApp);
app.route('/crashes', crashesApp);

// OpenAPI documentation
app.doc('/doc', {
	openapi: '3.0.0',
	info: {
		title: 'CTD API',
		version: '1.0.0',
		description: 'Crash to Desktop Reporter - Self-hosted API',
	},
});

// Swagger UI
app.get('/docs', swaggerUI({ url: '/doc' }));

// Error handler
app.onError((err, c) => {
	console.error(err);
	return c.json(
		{
			error: {
				code: 'INTERNAL_ERROR',
				message: 'An unexpected error occurred',
			},
		},
		500,
	);
});

// 404 handler
app.notFound((c) => {
	return c.json({ error: { code: 'NOT_FOUND', message: 'Not found' } }, 404);
});

export { app };
export type AppType = typeof app;
