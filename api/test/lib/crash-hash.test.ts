import { describe, expect, it } from 'vitest';

import {
	computeCrashHash,
	isSystemModule,
	parseStackTrace,
} from '@/lib/crash-hash';

describe('isSystemModule', () => {
	it('should identify system modules', () => {
		expect(isSystemModule('ntdll.dll')).toBe(true);
		expect(isSystemModule('kernel32.dll')).toBe(true);
		expect(isSystemModule('KERNELBASE.DLL')).toBe(true);
	});

	it('should not flag game modules', () => {
		expect(isSystemModule('SkyrimSE.exe')).toBe(false);
		expect(isSystemModule('ENBSeries.dll')).toBe(false);
		expect(isSystemModule('SKSE64_1_6_1130.dll')).toBe(false);
	});
});

describe('parseStackTrace', () => {
	it('should parse Crash Logger format', () => {
		const trace = '[0] 0x7FF712345678 SkyrimSE.exe+0x12345';
		const frames = parseStackTrace(trace);

		expect(frames).toHaveLength(1);
		expect(frames[0].module).toBe('SkyrimSE.exe');
		expect(frames[0].offset).toBe('0x12345');
		expect(frames[0].isSystemFrame).toBe(false);
	});

	it('should parse .NET Script Framework format', () => {
		const trace = 'SkyrimSE.exe+12345';
		const frames = parseStackTrace(trace);

		expect(frames).toHaveLength(1);
		expect(frames[0].module).toBe('SkyrimSE.exe');
		expect(frames[0].offset).toBe('12345');
	});

	it('should parse multiple frames', () => {
		const trace = `[0] 0x7FF712345678 SkyrimSE.exe+0x12345
[1] 0x7FF712345679 ENBSeries.dll+0x67890
[2] 0x7FF712345680 ntdll.dll+0xAAAAA`;
		const frames = parseStackTrace(trace);

		expect(frames).toHaveLength(3);
		expect(frames[0].module).toBe('SkyrimSE.exe');
		expect(frames[1].module).toBe('ENBSeries.dll');
		expect(frames[2].module).toBe('ntdll.dll');
		expect(frames[2].isSystemFrame).toBe(true);
	});

	it('should handle empty stack trace', () => {
		expect(parseStackTrace('')).toEqual([]);
		expect(parseStackTrace('   ')).toEqual([]);
	});

	it('should skip unparseable lines', () => {
		const trace = `some random text
SkyrimSE.exe+12345
more garbage`;
		const frames = parseStackTrace(trace);

		expect(frames).toHaveLength(1);
		expect(frames[0].module).toBe('SkyrimSE.exe');
	});
});

describe('computeCrashHash', () => {
	it('should produce same hash for equivalent crashes', () => {
		const trace1 =
			'[0] 0x7FF712345678 SkyrimSE.exe+0x12345\n[1] 0x7FF712345679 ENBSeries.dll+0x67890';
		const trace2 =
			'[0] 0x7FF799999999 SkyrimSE.exe+0x12345\n[1] 0x7FF788888888 ENBSeries.dll+0x67890';

		expect(computeCrashHash(trace1)).toBe(computeCrashHash(trace2));
	});

	it('should produce different hash for different crashes', () => {
		const trace1 = 'SkyrimSE.exe+0x12345';
		const trace2 = 'SkyrimSE.exe+0x99999';

		expect(computeCrashHash(trace1)).not.toBe(computeCrashHash(trace2));
	});

	it('should exclude system frames', () => {
		const trace =
			'SkyrimSE.exe+0x12345\nntdll.dll+0x99999\nkernel32.dll+0x88888';
		const frames = parseStackTrace(trace);
		const nonSystem = frames.filter((f) => !f.isSystemFrame);

		expect(nonSystem).toHaveLength(1);
		expect(nonSystem[0].module).toBe('SkyrimSE.exe');
	});

	it('should handle empty stack trace', () => {
		const hash = computeCrashHash('');
		expect(hash).toHaveLength(16);
	});

	it('should be case-insensitive for modules', () => {
		const trace1 = 'SkyrimSE.exe+0x12345';
		const trace2 = 'skyrimse.exe+0x12345';

		expect(computeCrashHash(trace1)).toBe(computeCrashHash(trace2));
	});

	it('should produce 16-character hex hash', () => {
		const hash = computeCrashHash('SkyrimSE.exe+0x12345');

		expect(hash).toHaveLength(16);
		expect(hash).toMatch(/^[0-9a-f]+$/);
	});

	it('should only use top 10 frames', () => {
		const frames = Array.from(
			{ length: 15 },
			(_, i) => `Module${i}.dll+0x${i}`,
		).join('\n');

		const hash = computeCrashHash(frames);
		expect(hash).toHaveLength(16);
	});

	it('should fallback to full trace hash when only system frames', () => {
		const trace = 'ntdll.dll+0x12345\nkernel32.dll+0x67890';
		const hash = computeCrashHash(trace);

		expect(hash).toHaveLength(16);
	});
});
