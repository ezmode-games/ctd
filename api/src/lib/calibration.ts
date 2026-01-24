/**
 * Confidence calibration for pattern matching predictions.
 *
 * Tracks predictions vs outcomes and measures calibration accuracy.
 * Uses this data to recalibrate confidence scores over time.
 */

export interface Prediction {
	id: string;
	predictedConfidence: number; // 0-100
	wasCorrect: boolean | null; // null = not yet validated
	validatedAt: Date | null;
}

export interface CalibrationBin {
	confidenceRange: [number, number];
	predictions: number;
	correct: number;
	accuracy: number; // correct / predictions
}

export interface CalibrationMetrics {
	bins: CalibrationBin[];
	expectedCalibrationError: number; // Mean |accuracy - confidence|
	maxCalibrationError: number; // Worst bin
	brierScore: number; // Proper scoring rule
	coverage: number; // % of predictions validated
	totalPredictions: number;
	validatedPredictions: number;
}

/**
 * Compute calibration metrics from predictions.
 */
export function computeCalibrationMetrics(
	predictions: Prediction[],
): CalibrationMetrics {
	const validated = predictions.filter((p) => p.wasCorrect !== null);
	const totalPredictions = predictions.length;
	const validatedPredictions = validated.length;

	// Create 10 bins: 0-10, 10-20, ..., 90-100
	const bins: CalibrationBin[] = [];
	for (let i = 0; i < 10; i++) {
		const lower = i * 10;
		const upper = (i + 1) * 10;

		const inBin = validated.filter(
			(p) => p.predictedConfidence >= lower && p.predictedConfidence < upper,
		);

		const correct = inBin.filter((p) => p.wasCorrect === true).length;
		const total = inBin.length;

		bins.push({
			confidenceRange: [lower, upper],
			predictions: total,
			correct,
			accuracy: total > 0 ? correct / total : 0,
		});
	}

	// Handle 100% confidence bin edge case
	const perfectConfidence = validated.filter(
		(p) => p.predictedConfidence === 100,
	);
	if (perfectConfidence.length > 0) {
		const lastBin = bins[bins.length - 1];
		lastBin.predictions += perfectConfidence.length;
		lastBin.correct += perfectConfidence.filter(
			(p) => p.wasCorrect === true,
		).length;
		lastBin.accuracy =
			lastBin.predictions > 0 ? lastBin.correct / lastBin.predictions : 0;
	}

	// Expected Calibration Error (ECE)
	// Weighted average of |accuracy - midpoint_confidence| per bin
	let eceSum = 0;
	let eceWeight = 0;

	for (const bin of bins) {
		if (bin.predictions > 0) {
			const midpoint = (bin.confidenceRange[0] + bin.confidenceRange[1]) / 2;
			const expectedAccuracy = midpoint / 100;
			eceSum += bin.predictions * Math.abs(bin.accuracy - expectedAccuracy);
			eceWeight += bin.predictions;
		}
	}

	const expectedCalibrationError = eceWeight > 0 ? eceSum / eceWeight : 0;

	// Maximum Calibration Error (MCE)
	let maxCalibrationError = 0;
	for (const bin of bins) {
		if (bin.predictions >= 5) {
			// Only consider bins with enough samples
			const midpoint = (bin.confidenceRange[0] + bin.confidenceRange[1]) / 2;
			const expectedAccuracy = midpoint / 100;
			const error = Math.abs(bin.accuracy - expectedAccuracy);
			maxCalibrationError = Math.max(maxCalibrationError, error);
		}
	}

	// Brier Score
	// Mean squared error of predictions (lower is better)
	let brierSum = 0;
	for (const p of validated) {
		const predicted = p.predictedConfidence / 100;
		const actual = p.wasCorrect ? 1 : 0;
		brierSum += (predicted - actual) ** 2;
	}
	const brierScore = validatedPredictions > 0 ? brierSum / validatedPredictions : 0;

	return {
		bins,
		expectedCalibrationError,
		maxCalibrationError,
		brierScore,
		coverage: totalPredictions > 0 ? validatedPredictions / totalPredictions : 0,
		totalPredictions,
		validatedPredictions,
	};
}

/**
 * Detect calibration drift by comparing recent metrics to baseline.
 */
export interface DriftAlert {
	severity: 'low' | 'medium' | 'high';
	message: string;
	recommendation: string;
	eceChange: number;
	brierChange: number;
}

export function detectDrift(
	recent: CalibrationMetrics,
	baseline: CalibrationMetrics,
): DriftAlert | null {
	// Need enough data in both sets
	if (recent.validatedPredictions < 20 || baseline.validatedPredictions < 20) {
		return null;
	}

	const eceChange = recent.expectedCalibrationError - baseline.expectedCalibrationError;
	const brierChange = recent.brierScore - baseline.brierScore;

	// Check for significant degradation
	if (eceChange > 0.15 || brierChange > 0.1) {
		return {
			severity: 'high',
			message: 'Pattern matching confidence is significantly miscalibrated',
			recommendation: 'Recalibrate immediately using recent validated predictions',
			eceChange,
			brierChange,
		};
	}

	if (eceChange > 0.1 || brierChange > 0.05) {
		return {
			severity: 'medium',
			message: 'Pattern matching confidence shows degradation',
			recommendation: 'Consider recalibrating using recent predictions',
			eceChange,
			brierChange,
		};
	}

	if (eceChange > 0.05) {
		return {
			severity: 'low',
			message: 'Minor calibration drift detected',
			recommendation: 'Monitor calibration metrics',
			eceChange,
			brierChange,
		};
	}

	return null;
}

/**
 * Platt scaling for recalibration.
 *
 * Fits a sigmoid function to map raw confidence to calibrated confidence.
 * Uses gradient descent to find optimal parameters.
 */
export interface PlattParameters {
	a: number; // Slope
	b: number; // Intercept
}

export function fitPlattScaling(predictions: Prediction[]): PlattParameters {
	const validated = predictions.filter((p) => p.wasCorrect !== null);

	if (validated.length < 10) {
		// Not enough data, return identity mapping
		return { a: 1, b: 0 };
	}

	// Simple gradient descent for logistic regression
	let a = 1;
	let b = 0;
	const learningRate = 0.01;
	const iterations = 1000;

	for (let iter = 0; iter < iterations; iter++) {
		let gradA = 0;
		let gradB = 0;

		for (const p of validated) {
			const x = p.predictedConfidence / 100;
			const y = p.wasCorrect ? 1 : 0;

			// Sigmoid: 1 / (1 + exp(-(a*x + b)))
			const logit = a * x + b;
			const pred = 1 / (1 + Math.exp(-logit));

			// Gradient of log loss
			const error = pred - y;
			gradA += error * x;
			gradB += error;
		}

		a -= (learningRate * gradA) / validated.length;
		b -= (learningRate * gradB) / validated.length;
	}

	return { a, b };
}

/**
 * Apply Platt scaling to recalibrate a confidence score.
 */
export function applyPlattScaling(
	confidence: number,
	params: PlattParameters,
): number {
	const x = confidence / 100;
	const logit = params.a * x + params.b;
	const calibrated = 1 / (1 + Math.exp(-logit));

	return Math.round(calibrated * 100);
}

/**
 * Isotonic regression for recalibration.
 *
 * Fits a monotonic (non-decreasing) function to the data.
 * More flexible than Platt scaling but requires more data.
 */
export interface IsotonicMapping {
	breakpoints: Array<{ input: number; output: number }>;
}

export function fitIsotonicRegression(predictions: Prediction[]): IsotonicMapping {
	const validated = predictions
		.filter((p) => p.wasCorrect !== null)
		.map((p) => ({
			x: p.predictedConfidence,
			y: p.wasCorrect ? 100 : 0,
		}))
		.sort((a, b) => a.x - b.x);

	if (validated.length < 10) {
		return { breakpoints: [{ input: 0, output: 0 }, { input: 100, output: 100 }] };
	}

	// Pool Adjacent Violators (PAV) algorithm
	const blocks: Array<{ sum: number; count: number; minX: number; maxX: number }> = [];

	for (const point of validated) {
		blocks.push({
			sum: point.y,
			count: 1,
			minX: point.x,
			maxX: point.x,
		});

		// Merge blocks that violate isotonicity
		while (blocks.length > 1) {
			const last = blocks[blocks.length - 1];
			const prev = blocks[blocks.length - 2];

			const lastMean = last.sum / last.count;
			const prevMean = prev.sum / prev.count;

			if (prevMean <= lastMean) {
				break; // No violation
			}

			// Merge blocks
			blocks.pop();
			prev.sum += last.sum;
			prev.count += last.count;
			prev.maxX = last.maxX;
		}
	}

	// Create breakpoints from blocks
	const breakpoints: Array<{ input: number; output: number }> = [];

	for (const block of blocks) {
		const mean = block.sum / block.count;
		if (breakpoints.length === 0 || block.minX > breakpoints[breakpoints.length - 1].input) {
			breakpoints.push({ input: block.minX, output: mean });
		}
		if (block.maxX > block.minX) {
			breakpoints.push({ input: block.maxX, output: mean });
		}
	}

	// Ensure we cover 0-100 range
	if (breakpoints.length === 0 || breakpoints[0].input > 0) {
		breakpoints.unshift({ input: 0, output: 0 });
	}
	if (breakpoints[breakpoints.length - 1].input < 100) {
		breakpoints.push({ input: 100, output: 100 });
	}

	return { breakpoints };
}

/**
 * Apply isotonic mapping to recalibrate a confidence score.
 */
export function applyIsotonicMapping(
	confidence: number,
	mapping: IsotonicMapping,
): number {
	const { breakpoints } = mapping;

	// Find surrounding breakpoints
	let lower = breakpoints[0];
	let upper = breakpoints[breakpoints.length - 1];

	for (let i = 0; i < breakpoints.length - 1; i++) {
		if (breakpoints[i].input <= confidence && breakpoints[i + 1].input >= confidence) {
			lower = breakpoints[i];
			upper = breakpoints[i + 1];
			break;
		}
	}

	// Linear interpolation
	if (upper.input === lower.input) {
		return Math.round(lower.output);
	}

	const t = (confidence - lower.input) / (upper.input - lower.input);
	const calibrated = lower.output + t * (upper.output - lower.output);

	return Math.round(Math.max(0, Math.min(100, calibrated)));
}
