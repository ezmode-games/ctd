import { describe, expect, it } from 'vitest';

import { app } from '../../src/app';

describe('POST /api/crash-reports', () => {
	it('should create a crash report', async () => {
		const res = await app.request('/api/crash-reports', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				gameId: 'skyrim-se',
				stackTrace: 'SkyrimSE.exe+0x12345\nENBSeries.dll+0x67890',
				gameVersion: '1.6.1130',
				loadOrderJson:
					'[{"name":"Unofficial Skyrim Special Edition Patch.esp","enabled":true}]',
				pluginCount: 1,
				crashedAt: Date.now(),
			}),
		});

		expect(res.status).toBe(201);
		const json = await res.json();
		expect(json.id).toBeDefined();
		expect(json.shareToken).toBeDefined();
		expect(json.shareToken).toHaveLength(16);
	});

	it('should accept optional crashHash', async () => {
		const res = await app.request('/api/crash-reports', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				gameId: 'skyrim-se',
				stackTrace: 'crash',
				crashHash: 'abc123def456',
				gameVersion: '1.6.1130',
				loadOrderJson: '[]',
				pluginCount: 0,
				crashedAt: Date.now(),
			}),
		});

		expect(res.status).toBe(201);
	});

	it('should reject invalid loadOrderJson', async () => {
		const res = await app.request('/api/crash-reports', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				gameId: 'skyrim-se',
				stackTrace: 'crash',
				gameVersion: '1.6.1130',
				loadOrderJson: 'not json',
				pluginCount: 0,
				crashedAt: Date.now(),
			}),
		});

		expect(res.status).toBe(422);
	});

	it('should reject missing required fields', async () => {
		const res = await app.request('/api/crash-reports', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				gameId: 'skyrim-se',
			}),
		});

		expect(res.status).toBe(422);
	});

	it('should accept optional fields', async () => {
		const res = await app.request('/api/crash-reports', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				gameId: 'skyrim-se',
				stackTrace: 'SkyrimSE.exe+0x12345',
				gameVersion: '1.6.1130',
				loadOrderJson: '[]',
				pluginCount: 0,
				crashedAt: Date.now(),
				exceptionCode: 'EXCEPTION_ACCESS_VIOLATION',
				exceptionAddress: '0x7FF712345678',
				faultingModule: 'SkyrimSE.exe',
				scriptExtenderVersion: '2.2.3',
				osVersion: 'Windows 10 22H2',
				notes: 'Crashed after fast travel',
			}),
		});

		expect(res.status).toBe(201);
	});
});
