import { describe, expect, it } from 'vitest';

import { app } from '../src/app';

describe('API basics', () => {
	it('GET /health returns ok', async () => {
		const res = await app.request('/health');

		expect(res.status).toBe(200);
		const json = await res.json();
		expect(json.status).toBe('ok');
		expect(json.version).toBe('0.1.0');
	});

	it('GET /unknown returns 404', async () => {
		const res = await app.request('/unknown');

		expect(res.status).toBe(404);
		const json = await res.json();
		expect(json.error.code).toBe('NOT_FOUND');
	});
});
