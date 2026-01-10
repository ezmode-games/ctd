import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { requestId } from 'hono/request-id';

import { apiKeys } from '@/routes/api-keys';
import { config } from '@/routes/config';
import { crashes } from '@/routes/crashes';
import { docs } from '@/routes/docs';

const app = new Hono();

// Middleware
app.use('*', cors());
app.use('*', requestId());

// Health check
app.get('/health', (c) => {
	return c.json({ status: 'ok', version: '0.1.0' });
});

// Routes
app.route('/api-keys', apiKeys);
app.route('/config', config);
app.route('/crashes', crashes);
app.route('/docs', docs);

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
