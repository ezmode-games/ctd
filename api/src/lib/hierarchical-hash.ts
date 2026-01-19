/**
 * Hierarchical Crash Signatures
 *
 * Generates multiple hash levels for confidence-based matching.
 * Instead of a single exact-match hash, we create 5 levels from broad to specific.
 *
 * Level 1 (broadest): faulting_module only
 * Level 2: faulting_module + exception_code
 * Level 3: faulting_module + top_frame (module+offset)
 * Level 4: faulting_module + top_3_frames
 * Level 5 (exact): top_10_frames normalized (current behavior)
 *
 * Match at the most specific level that has enough data for target confidence.
 * New crash with N=1? Match at L1-L2. N=50? Match at L4-L5.
 *
 * @see https://github.com/ezmode-games/ctd/issues/60
 */

import { createHash } from 'node:crypto';
import { type StackFrame, parseStackTrace } from './crash-hash';

/**
 * Hierarchical signature with all 5 levels
 */
export interface HierarchicalSignature {
	/** Level 1: faulting_module only (broadest) */
	l1: string;
	/** Level 2: faulting_module + exception_code */
	l2: string;
	/** Level 3: faulting_module + top_frame */
	l3: string;
	/** Level 4: faulting_module + top_3_frames */
	l4: string;
	/** Level 5: top_10_frames normalized (most specific, backwards compatible) */
	l5: string;
	/** Metadata about what went into each hash */
	meta: SignatureMetadata;
}

export interface SignatureMetadata {
	faultingModule: string | null;
	exceptionCode: string | null;
	gameFrameCount: number;
	topFrames: Array<{ module: string; offset: string }>;
}

/**
 * SHA-256 hash truncated to 16 hex chars
 */
function hash(input: string): string {
	return createHash('sha256').update(input).digest('hex').slice(0, 16);
}

/**
 * Normalize a module name for hashing
 */
function normalizeModule(module: string): string {
	return module.toLowerCase().replace(/\.dll$|\.exe$/i, '');
}

/**
 * Get game frames (non-system) from parsed stack trace
 */
function getGameFrames(frames: StackFrame[]): StackFrame[] {
	return frames.filter((f) => !f.isSystemFrame);
}

/**
 * Determine the faulting module from stack trace
 *
 * The faulting module is typically the first non-system frame,
 * or the first frame if all are system frames.
 */
function determineFaultingModule(frames: StackFrame[]): string | null {
	const gameFrames = getGameFrames(frames);
	if (gameFrames.length > 0 && gameFrames[0]) {
		return gameFrames[0].module;
	}
	// Fallback to first frame if no game frames
	if (frames.length > 0 && frames[0]) {
		return frames[0].module;
	}
	return null;
}

/**
 * Compute hierarchical crash signatures from a stack trace
 *
 * @param stackTrace - Raw stack trace text
 * @param exceptionCode - Optional exception code (e.g., "0xC0000005")
 * @param faultingModuleOverride - Optional override for faulting module
 * @returns HierarchicalSignature with all 5 levels
 *
 * @example
 * ```typescript
 * const sig = computeHierarchicalSignature(stackTrace, "0xC0000005");
 *
 * // Store all levels
 * await db.insert(crashReport).values({
 *   crashHash: sig.l5,  // Backwards compatible
 *   l1Hash: sig.l1,
 *   l2Hash: sig.l2,
 *   l3Hash: sig.l3,
 *   l4Hash: sig.l4,
 * });
 *
 * // Match at appropriate level based on data
 * const pattern = await findPatternAtBestLevel(sig, minConfidence);
 * ```
 */
export function computeHierarchicalSignature(
	stackTrace: string,
	exceptionCode?: string,
	faultingModuleOverride?: string,
): HierarchicalSignature {
	const frames = parseStackTrace(stackTrace);
	const gameFrames = getGameFrames(frames);
	const faultingModule = faultingModuleOverride ?? determineFaultingModule(frames);
	const normalizedFaulting = faultingModule ? normalizeModule(faultingModule) : 'unknown';
	const normalizedExCode = exceptionCode?.toLowerCase() ?? 'unknown';

	// Build frame strings for each level
	const topFrames = gameFrames.slice(0, 10).map((f) => ({
		module: f.module,
		offset: f.offset,
	}));

	const frameString = (count: number): string => {
		return gameFrames
			.slice(0, count)
			.map((f) => `${normalizeModule(f.module)}+${f.offset}`)
			.join('|');
	};

	// Level 1: faulting_module only
	const l1Input = normalizedFaulting;

	// Level 2: faulting_module + exception_code
	const l2Input = `${normalizedFaulting}:${normalizedExCode}`;

	// Level 3: faulting_module + top_frame
	const topFrame = gameFrames[0];
	const l3Input = topFrame
		? `${normalizedFaulting}:${normalizeModule(topFrame.module)}+${topFrame.offset}`
		: l2Input;

	// Level 4: faulting_module + top_3_frames
	const top3 = frameString(3);
	const l4Input = top3 ? `${normalizedFaulting}:${top3}` : l3Input;

	// Level 5: top_10_frames normalized (backwards compatible with computeCrashHash)
	const top10 = frameString(10);
	const l5Input = top10 || stackTrace; // Fallback to full trace if no game frames

	return {
		l1: hash(l1Input),
		l2: hash(l2Input),
		l3: hash(l3Input),
		l4: hash(l4Input),
		l5: hash(l5Input),
		meta: {
			faultingModule,
			exceptionCode: exceptionCode ?? null,
			gameFrameCount: gameFrames.length,
			topFrames,
		},
	};
}

/**
 * Match level information for API responses
 */
export interface SignatureMatch {
	/** Which level matched (1-5, or 0 for no match) */
	level: 0 | 1 | 2 | 3 | 4 | 5;
	/** Hash that matched */
	hash: string;
	/** Human-readable description of match specificity */
	description: string;
}

/**
 * Get the hash at a specific level
 */
export function getHashAtLevel(sig: HierarchicalSignature, level: 1 | 2 | 3 | 4 | 5): string {
	const levels: Record<1 | 2 | 3 | 4 | 5, string> = {
		1: sig.l1,
		2: sig.l2,
		3: sig.l3,
		4: sig.l4,
		5: sig.l5,
	};
	return levels[level];
}

/**
 * Get description for a match level
 */
export function getLevelDescription(level: 0 | 1 | 2 | 3 | 4 | 5): string {
	const descriptions: Record<0 | 1 | 2 | 3 | 4 | 5, string> = {
		0: 'No match found',
		1: 'Same faulting module (broad category)',
		2: 'Same module and exception code',
		3: 'Same crash location (module + top frame)',
		4: 'Similar crash signature (top 3 frames)',
		5: 'Exact match (top 10 frames)',
	};
	return descriptions[level];
}

/**
 * Minimum occurrences needed at each level for confident matching
 *
 * Higher levels (more specific) need fewer samples because the match is more certain.
 * Lower levels (broader) need more samples to be confident it's the same issue.
 */
export const MIN_OCCURRENCES_FOR_CONFIDENCE: Record<1 | 2 | 3 | 4 | 5, number> = {
	1: 50, // Very broad - need lots of data
	2: 30, // Still broad
	3: 15, // Getting specific
	4: 5, // Pretty specific
	5: 1, // Exact match - one is enough
};
