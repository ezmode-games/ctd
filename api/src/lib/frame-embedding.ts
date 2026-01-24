/**
 * Frame embedding for fuzzy crash matching.
 *
 * Converts stack frames into vectors for similarity search.
 * Uses a simple but effective approach: TF-IDF-like weighting
 * of module+offset tokens.
 */

import type { StackFrame } from './crash-hash';
import { parseStackTrace } from './crash-hash';

// Embedding dimension (tunable)
const VECTOR_DIM = 64;

// Prime numbers for hashing (for consistent bucket assignment)
const HASH_PRIMES = [
	31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107,
	109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191,
	193, 197, 199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271,
	277, 281, 283, 293, 307, 311, 313, 317, 331, 337, 347, 349, 353, 359, 367,
	373,
];

/**
 * Simple string hash function.
 */
function hashString(s: string): number {
	let hash = 0;
	for (let i = 0; i < s.length; i++) {
		const char = s.charCodeAt(i);
		hash = (hash << 5) - hash + char;
		hash = hash & hash; // Convert to 32-bit integer
	}
	return Math.abs(hash);
}

/**
 * Normalize a module name for embedding.
 * Removes version numbers, common suffixes, etc.
 */
function normalizeModule(module: string): string {
	return module
		.toLowerCase()
		.replace(/\.dll$/i, '')
		.replace(/\.exe$/i, '')
		.replace(/\d+$/, '') // Remove trailing numbers (versions)
		.replace(/_[a-f0-9]{8}$/i, ''); // Remove hash suffixes
}

/**
 * Extract offset bucket for embedding.
 * Groups similar offsets together (within 0x1000 range).
 */
function offsetBucket(offset: string): string {
	const num = Number.parseInt(offset.replace(/^0x/i, ''), 16);
	if (Number.isNaN(num)) return 'unknown';

	// Bucket by 4KB pages
	const bucket = Math.floor(num / 0x1000);
	return `${bucket}`;
}

/**
 * Generate embedding vector from stack frames.
 *
 * Uses locality-sensitive hashing to create a fixed-size vector
 * that captures the stack trace structure.
 */
export function embedFrames(frames: StackFrame[]): number[] {
	const vector = new Array(VECTOR_DIM).fill(0);

	// Weight frames by position (top of stack more important)
	const maxWeight = 10;

	for (let i = 0; i < Math.min(frames.length, 20); i++) {
		const frame = frames[i];
		const weight = maxWeight - i * 0.5; // Decreasing weight

		// Skip system frames but still use them for context
		const frameWeight = frame.isSystemFrame ? weight * 0.2 : weight;

		// Hash module name into vector buckets
		const normalizedModule = normalizeModule(frame.module);
		const moduleHash = hashString(normalizedModule);

		// Add to multiple buckets (MinHash-like)
		for (let j = 0; j < 4; j++) {
			const bucket = (moduleHash * HASH_PRIMES[j]) % VECTOR_DIM;
			vector[bucket] += frameWeight;
		}

		// Hash module+offset for more specificity
		const offsetBkt = offsetBucket(frame.offset);
		const fullHash = hashString(`${normalizedModule}:${offsetBkt}`);

		for (let j = 0; j < 2; j++) {
			const bucket = (fullHash * HASH_PRIMES[j + 4]) % VECTOR_DIM;
			vector[bucket] += frameWeight * 0.5;
		}

		// Hash just the offset bucket (for cross-module patterns)
		const offsetHash = hashString(offsetBkt);
		const offsetBucket2 = (offsetHash * HASH_PRIMES[6]) % VECTOR_DIM;
		vector[offsetBucket2] += frameWeight * 0.25;
	}

	// L2 normalize
	const norm = Math.sqrt(vector.reduce((sum, v) => sum + v * v, 0));
	if (norm > 0) {
		for (let i = 0; i < vector.length; i++) {
			vector[i] = vector[i] / norm;
		}
	}

	return vector;
}

/**
 * Embed a stack trace string.
 */
export function embedStackTrace(stackTrace: string): number[] {
	const frames = parseStackTrace(stackTrace);
	return embedFrames(frames);
}

/**
 * Compute cosine similarity between two vectors.
 */
export function cosineSimilarity(a: number[], b: number[]): number {
	if (a.length !== b.length) {
		throw new Error('Vectors must have same dimension');
	}

	let dotProduct = 0;
	let normA = 0;
	let normB = 0;

	for (let i = 0; i < a.length; i++) {
		dotProduct += a[i] * b[i];
		normA += a[i] * a[i];
		normB += b[i] * b[i];
	}

	const denominator = Math.sqrt(normA) * Math.sqrt(normB);
	if (denominator === 0) return 0;

	return dotProduct / denominator;
}

/**
 * Find similar crashes from a set of embeddings.
 */
export function findSimilar(
	queryVector: number[],
	candidates: Array<{ id: string; vector: number[] }>,
	options: {
		topK?: number;
		minSimilarity?: number;
	} = {},
): Array<{ id: string; similarity: number }> {
	const { topK = 10, minSimilarity = 0.5 } = options;

	const results = candidates
		.map((c) => ({
			id: c.id,
			similarity: cosineSimilarity(queryVector, c.vector),
		}))
		.filter((r) => r.similarity >= minSimilarity)
		.sort((a, b) => b.similarity - a.similarity)
		.slice(0, topK);

	return results;
}

/**
 * Classify similarity level.
 */
export function classifySimilarity(
	similarity: number,
): 'exact' | 'similar' | 'related' | 'category' | 'unrelated' {
	if (similarity >= 0.99) return 'exact';
	if (similarity >= 0.85) return 'similar';
	if (similarity >= 0.7) return 'related';
	if (similarity >= 0.5) return 'category';
	return 'unrelated';
}

/**
 * Serialize vector to JSON for storage.
 */
export function serializeVector(vector: number[]): string {
	// Round to 4 decimal places to save space
	return JSON.stringify(vector.map((v) => Math.round(v * 10000) / 10000));
}

/**
 * Deserialize vector from JSON.
 */
export function deserializeVector(json: string): number[] {
	return JSON.parse(json);
}

export { VECTOR_DIM };
