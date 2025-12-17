import { describe, expect, it } from 'vitest';

import { app } from '../src/app';
import { mockCrashReport } from './fixtures';

describe('POST /crashes', () => {
	it('creates a crash report', async () => {
		const report = mockCrashReport();

		const res = await app.request('/crashes', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(report),
		});

		expect(res.status).toBe(201);
		const json = await res.json();
		expect(json.id).toBeDefined();
		expect(json.shareToken).toBeDefined();
		expect(json.shareToken).toHaveLength(16);
	});

	it('accepts optional crashHash', async () => {
		const report = mockCrashReport();
		report.crashHash = 'custom-hash-123';

		const res = await app.request('/crashes', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(report),
		});

		expect(res.status).toBe(201);
	});

	it('rejects invalid loadOrderJson', async () => {
		const report = mockCrashReport();
		report.loadOrderJson = 'not json';

		const res = await app.request('/crashes', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(report),
		});

		expect(res.status).toBe(422);
	});

	it('rejects missing required fields', async () => {
		const res = await app.request('/crashes', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ gameId: 'skyrim-se' }),
		});

		expect(res.status).toBe(422);
	});
});

describe('GET /crashes/:id', () => {
	it('returns crash report with valid share token', async () => {
		// First create a report
		const report = mockCrashReport();
		const createRes = await app.request('/crashes', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(report),
		});
		const { id, shareToken } = await createRes.json();

		// Then fetch it with token
		const res = await app.request(`/crashes/${id}?token=${shareToken}`);

		expect(res.status).toBe(200);
		const json = await res.json();
		expect(json.id).toBe(id);
		expect(json.gameId).toBe(report.gameId);
		expect(json.loadOrder).toBeInstanceOf(Array);
	});

	it('returns 404 for private report without token', async () => {
		// Create a report
		const report = mockCrashReport();
		const createRes = await app.request('/crashes', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(report),
		});
		const { id } = await createRes.json();

		// Try to fetch without token
		const res = await app.request(`/crashes/${id}`);

		expect(res.status).toBe(404);
	});

	it('returns 404 for non-existent report', async () => {
		const res = await app.request('/crashes/nonexistent');

		expect(res.status).toBe(404);
	});
});
