/**
 * Mod correlation scoring using lift and PMI statistics.
 *
 * Computes how strongly a mod correlates with a crash pattern.
 * Lift > 1 means the mod appears more often in crashes than expected.
 */

export interface ModOccurrence {
	modName: string;
	modFingerprint?: string;
}

export interface CorrelationResult {
	modName: string;
	modFingerprint?: string;

	// Raw counts
	crashesWithMod: number;
	crashesWithoutMod: number;
	totalCrashes: number;
	baseRatePct: number; // 0-100, how common is this mod overall

	// Computed metrics
	lift: number; // P(crash|mod) / P(crash) - values > 1 indicate correlation
	pmi: number; // Pointwise mutual information
	correlationScore: number; // 0-100 normalized score

	// Confidence (Wilson score interval)
	confidence: number; // 0-100
	lowerBound: number;
	upperBound: number;
}

/**
 * Extract mod names from a load order JSON string.
 */
export function extractModNames(loadOrderJson: string): ModOccurrence[] {
	try {
		const loadOrder = JSON.parse(loadOrderJson);
		if (!Array.isArray(loadOrder)) return [];

		return loadOrder
			.filter((entry: unknown) => {
				if (typeof entry === 'string') return true;
				if (typeof entry === 'object' && entry !== null) {
					const obj = entry as Record<string, unknown>;
					return typeof obj.name === 'string';
				}
				return false;
			})
			.map((entry: unknown) => {
				if (typeof entry === 'string') {
					return { modName: entry };
				}
				const obj = entry as Record<string, unknown>;
				return {
					modName: obj.name as string,
					modFingerprint: obj.fileHash as string | undefined,
				};
			});
	} catch {
		return [];
	}
}

/**
 * Compute lift: P(pattern|mod) / P(pattern|no mod)
 *
 * - lift = 1.0: No correlation (mod has no effect)
 * - lift > 1.0: Positive correlation (mod increases crash likelihood)
 * - lift < 1.0: Negative correlation (mod is protective)
 */
export function computeLift(
	crashesWithMod: number,
	crashesWithoutMod: number,
	modBaseRate: number, // 0-1, proportion of all crashes that have this mod
): number {
	const totalCrashes = crashesWithMod + crashesWithoutMod;
	if (totalCrashes === 0 || modBaseRate <= 0 || modBaseRate >= 1) {
		return 1.0; // No data or invalid base rate
	}

	// P(pattern|mod) = crashesWithMod / (total crashes with mod in population)
	// For now, we estimate using the pattern's crash data
	const pPatternGivenMod = crashesWithMod / (totalCrashes * modBaseRate);

	// P(pattern|no mod)
	const pPatternGivenNoMod =
		crashesWithoutMod / (totalCrashes * (1 - modBaseRate));

	if (pPatternGivenNoMod === 0) {
		return crashesWithMod > 0 ? 10.0 : 1.0; // Cap at 10x if denominator is 0
	}

	return Math.min(10.0, Math.max(0.1, pPatternGivenMod / pPatternGivenNoMod));
}

/**
 * Compute PMI (Pointwise Mutual Information)
 *
 * PMI = log2(P(pattern, mod) / (P(pattern) * P(mod)))
 *
 * - PMI = 0: No correlation
 * - PMI > 0: Positive correlation
 * - PMI < 0: Negative correlation
 */
export function computePMI(
	crashesWithMod: number,
	totalCrashes: number,
	modBaseRate: number,
): number {
	if (totalCrashes === 0 || modBaseRate <= 0 || crashesWithMod === 0) {
		return 0;
	}

	const pJoint = crashesWithMod / totalCrashes;
	const pPattern = 1; // We're conditioning on the pattern
	const pMod = modBaseRate;

	const pmi = Math.log2(pJoint / (pPattern * pMod));

	// Clamp to reasonable range
	return Math.max(-5, Math.min(5, pmi));
}

/**
 * Wilson score confidence interval for a proportion.
 * More accurate than normal approximation for small samples.
 */
export function wilsonScore(
	successes: number,
	total: number,
	confidence = 0.95,
): { lower: number; upper: number } {
	if (total === 0) {
		return { lower: 0, upper: 1 };
	}

	// Z-score for confidence level (1.96 for 95%)
	const z = confidence === 0.95 ? 1.96 : 1.645; // 95% or 90%

	const p = successes / total;
	const n = total;

	const denominator = 1 + (z * z) / n;
	const center = p + (z * z) / (2 * n);
	const margin = z * Math.sqrt((p * (1 - p) + (z * z) / (4 * n)) / n);

	return {
		lower: Math.max(0, (center - margin) / denominator),
		upper: Math.min(1, (center + margin) / denominator),
	};
}

/**
 * Convert lift to a 0-100 correlation score.
 *
 * Score interpretation:
 * - 0-20: Protective (lift < 0.5)
 * - 20-40: No significant correlation (lift 0.5-1.5)
 * - 40-60: Weak positive (lift 1.5-2.5)
 * - 60-80: Moderate positive (lift 2.5-5)
 * - 80-100: Strong positive (lift > 5)
 */
export function liftToScore(lift: number): number {
	if (lift < 0.5) {
		// Protective: 0-20
		return Math.round(20 * (lift / 0.5));
	}
	if (lift < 1.5) {
		// No significant correlation: 20-40
		return Math.round(20 + 20 * ((lift - 0.5) / 1.0));
	}
	if (lift < 2.5) {
		// Weak positive: 40-60
		return Math.round(40 + 20 * ((lift - 1.5) / 1.0));
	}
	if (lift < 5.0) {
		// Moderate positive: 60-80
		return Math.round(60 + 20 * ((lift - 2.5) / 2.5));
	}
	// Strong positive: 80-100
	return Math.round(80 + 20 * Math.min(1, (lift - 5.0) / 5.0));
}

/**
 * Compute confidence score based on sample size.
 * Uses Wilson score interval width.
 */
export function computeConfidence(
	crashesWithMod: number,
	totalCrashes: number,
): number {
	if (totalCrashes < 5) {
		return 0; // Not enough data
	}

	const interval = wilsonScore(crashesWithMod, totalCrashes);
	const width = interval.upper - interval.lower;

	// Narrower interval = higher confidence
	// Width of 0 = 100% confidence, width of 1 = 0% confidence
	const confidence = Math.round(100 * (1 - width));

	return Math.max(0, Math.min(100, confidence));
}

/**
 * Compute full correlation analysis for a mod against a pattern.
 */
export function computeModCorrelation(
	modName: string,
	crashesWithMod: number,
	crashesWithoutMod: number,
	modBaseRate: number, // 0-1
	modFingerprint?: string,
): CorrelationResult {
	const totalCrashes = crashesWithMod + crashesWithoutMod;

	const lift = computeLift(crashesWithMod, crashesWithoutMod, modBaseRate);
	const pmi = computePMI(crashesWithMod, totalCrashes, modBaseRate);
	const correlationScore = liftToScore(lift);
	const confidence = computeConfidence(crashesWithMod, totalCrashes);
	const interval = wilsonScore(crashesWithMod, totalCrashes);

	return {
		modName,
		modFingerprint,
		crashesWithMod,
		crashesWithoutMod,
		totalCrashes,
		baseRatePct: Math.round(modBaseRate * 100),
		lift,
		pmi,
		correlationScore,
		confidence,
		lowerBound: interval.lower,
		upperBound: interval.upper,
	};
}

/**
 * Determine if a correlation is statistically significant.
 */
export function isSignificant(result: CorrelationResult): boolean {
	// Need at least 10 crashes and confidence > 50
	if (result.totalCrashes < 10 || result.confidence < 50) {
		return false;
	}

	// Lift must be outside 0.7-1.3 range (meaningfully different from 1.0)
	if (result.lift > 0.7 && result.lift < 1.3) {
		return false;
	}

	return true;
}
