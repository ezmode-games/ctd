import { describe, expect, it } from 'vitest';

import { app } from '../src/app';

describe('API scaffold', () => {
	it('GET / returns API info', async () => {
		const res = await app.request('/');

		expect(res.status).toBe(200);
		const json = await res.json();
		expect(json).toEqual({ message: 'CTD API' });
	});

	it('GET /health returns health check', async () => {
		const res = await app.request('/health');

		expect(res.status).toBe(200);
		const json = await res.json();
		expect(json.status).toBe('ok');
		expect(json.version).toBe('0.1.0');
	});

	it('GET /docs returns OpenAPI spec', async () => {
		const res = await app.request('/docs');

		expect(res.status).toBe(200);
		const json = await res.json();
		expect(json.openapi).toBe('3.1.0');
		expect(json.info.title).toBe('CTD API');
	});

	it('GET /unknown returns 404', async () => {
		const res = await app.request('/unknown');

		expect(res.status).toBe(404);
	});
});
