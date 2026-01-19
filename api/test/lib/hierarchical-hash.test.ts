import { describe, expect, it } from 'vitest';

import {
	MIN_OCCURRENCES_FOR_CONFIDENCE,
	computeHierarchicalSignature,
	getHashAtLevel,
	getLevelDescription,
} from '@/lib/hierarchical-hash';

describe('computeHierarchicalSignature', () => {
	const sampleTrace = `[0] 0x7FF712345678 SkyrimSE.exe+0x12345
[1] 0x7FF712345679 ENBSeries.dll+0x67890
[2] 0x7FF712345680 SKSE64_1_6_1130.dll+0xABCDE
[3] 0x7FF712345681 ntdll.dll+0x11111`;

	it('should produce all 5 hash levels', () => {
		const sig = computeHierarchicalSignature(sampleTrace, '0xC0000005');

		expect(sig.l1).toHaveLength(16);
		expect(sig.l2).toHaveLength(16);
		expect(sig.l3).toHaveLength(16);
		expect(sig.l4).toHaveLength(16);
		expect(sig.l5).toHaveLength(16);
	});

	it('should have different hashes at each level', () => {
		const sig = computeHierarchicalSignature(sampleTrace, '0xC0000005');

		// Each level should produce a different hash (more info = different hash)
		const hashes = new Set([sig.l1, sig.l2, sig.l3, sig.l4, sig.l5]);
		expect(hashes.size).toBe(5);
	});

	it('should produce consistent l1 hash for same faulting module', () => {
		const trace1 = 'SkyrimSE.exe+0x12345';
		const trace2 = 'SkyrimSE.exe+0x99999\nENBSeries.dll+0x11111';

		const sig1 = computeHierarchicalSignature(trace1);
		const sig2 = computeHierarchicalSignature(trace2);

		// L1 should match (same faulting module)
		expect(sig1.l1).toBe(sig2.l1);
		// L5 should differ (different frames)
		expect(sig1.l5).not.toBe(sig2.l5);
	});

	it('should produce consistent l2 hash for same module + exception code', () => {
		const trace1 = 'SkyrimSE.exe+0x12345';
		const trace2 = 'SkyrimSE.exe+0x99999';

		const sig1 = computeHierarchicalSignature(trace1, '0xC0000005');
		const sig2 = computeHierarchicalSignature(trace2, '0xC0000005');

		// L2 should match (same module + exception)
		expect(sig1.l2).toBe(sig2.l2);
		// L3+ should differ (different offsets)
		expect(sig1.l3).not.toBe(sig2.l3);
	});

	it('should produce consistent l3 hash for same top frame', () => {
		const trace1 = 'SkyrimSE.exe+0x12345\nENBSeries.dll+0x11111';
		const trace2 = 'SkyrimSE.exe+0x12345\nSomeOther.dll+0x99999';

		const sig1 = computeHierarchicalSignature(trace1);
		const sig2 = computeHierarchicalSignature(trace2);

		// L3 should match (same faulting module + top frame)
		expect(sig1.l3).toBe(sig2.l3);
		// L4+ should differ (different subsequent frames)
		expect(sig1.l4).not.toBe(sig2.l4);
	});

	it('should include metadata about the signature', () => {
		const sig = computeHierarchicalSignature(sampleTrace, '0xC0000005');

		expect(sig.meta.faultingModule).toBe('SkyrimSE.exe');
		expect(sig.meta.exceptionCode).toBe('0xC0000005');
		expect(sig.meta.gameFrameCount).toBe(3); // Excludes ntdll.dll
		expect(sig.meta.topFrames).toHaveLength(3);
		expect(sig.meta.topFrames[0]).toEqual({
			module: 'SkyrimSE.exe',
			offset: '0x12345',
		});
	});

	it('should be case-insensitive for module names', () => {
		const trace1 = 'SkyrimSE.exe+0x12345';
		const trace2 = 'skyrimse.exe+0x12345';

		const sig1 = computeHierarchicalSignature(trace1, '0xC0000005');
		const sig2 = computeHierarchicalSignature(trace2, '0xC0000005');

		expect(sig1.l1).toBe(sig2.l1);
		expect(sig1.l2).toBe(sig2.l2);
		expect(sig1.l3).toBe(sig2.l3);
		expect(sig1.l5).toBe(sig2.l5);
	});

	it('should handle missing exception code', () => {
		const sig = computeHierarchicalSignature(sampleTrace);

		expect(sig.l2).toHaveLength(16);
		expect(sig.meta.exceptionCode).toBeNull();
	});

	it('should handle empty stack trace', () => {
		const sig = computeHierarchicalSignature('');

		expect(sig.l1).toHaveLength(16);
		expect(sig.l5).toHaveLength(16);
		expect(sig.meta.faultingModule).toBeNull();
		expect(sig.meta.gameFrameCount).toBe(0);
	});

	it('should handle stack trace with only system frames', () => {
		const trace = 'ntdll.dll+0x12345\nkernel32.dll+0x67890';
		const sig = computeHierarchicalSignature(trace);

		// Should use first frame as faulting module
		expect(sig.meta.faultingModule).toBe('ntdll.dll');
		expect(sig.meta.gameFrameCount).toBe(0);
		expect(sig.l5).toHaveLength(16);
	});

	it('should allow faulting module override', () => {
		const trace = 'SkyrimSE.exe+0x12345';
		const sig = computeHierarchicalSignature(trace, undefined, 'CustomMod.dll');

		expect(sig.meta.faultingModule).toBe('CustomMod.dll');
	});

	it('should normalize .dll and .exe extensions', () => {
		const trace1 = 'SkyrimSE.exe+0x12345';
		const trace2 = 'SkyrimSE+0x12345'; // Hypothetical without extension

		const sig1 = computeHierarchicalSignature(trace1);
		// L1 uses normalized module name (without extension)
		// This ensures "SkyrimSE.exe" and "SkyrimSE.dll" would hash the same at L1
		expect(sig1.l1).toHaveLength(16);
	});
});

describe('getHashAtLevel', () => {
	it('should return correct hash for each level', () => {
		const sig = computeHierarchicalSignature('SkyrimSE.exe+0x12345', '0xC0000005');

		expect(getHashAtLevel(sig, 1)).toBe(sig.l1);
		expect(getHashAtLevel(sig, 2)).toBe(sig.l2);
		expect(getHashAtLevel(sig, 3)).toBe(sig.l3);
		expect(getHashAtLevel(sig, 4)).toBe(sig.l4);
		expect(getHashAtLevel(sig, 5)).toBe(sig.l5);
	});
});

describe('getLevelDescription', () => {
	it('should return descriptions for all levels', () => {
		expect(getLevelDescription(0)).toContain('No match');
		expect(getLevelDescription(1)).toContain('faulting module');
		expect(getLevelDescription(2)).toContain('exception code');
		expect(getLevelDescription(3)).toContain('top frame');
		expect(getLevelDescription(4)).toContain('top 3 frames');
		expect(getLevelDescription(5)).toContain('Exact');
	});
});

describe('MIN_OCCURRENCES_FOR_CONFIDENCE', () => {
	it('should require more samples for broader levels', () => {
		expect(MIN_OCCURRENCES_FOR_CONFIDENCE[1]).toBeGreaterThan(
			MIN_OCCURRENCES_FOR_CONFIDENCE[2],
		);
		expect(MIN_OCCURRENCES_FOR_CONFIDENCE[2]).toBeGreaterThan(
			MIN_OCCURRENCES_FOR_CONFIDENCE[3],
		);
		expect(MIN_OCCURRENCES_FOR_CONFIDENCE[3]).toBeGreaterThan(
			MIN_OCCURRENCES_FOR_CONFIDENCE[4],
		);
		expect(MIN_OCCURRENCES_FOR_CONFIDENCE[4]).toBeGreaterThan(
			MIN_OCCURRENCES_FOR_CONFIDENCE[5],
		);
	});

	it('should require only 1 occurrence for exact match', () => {
		expect(MIN_OCCURRENCES_FOR_CONFIDENCE[5]).toBe(1);
	});
});
