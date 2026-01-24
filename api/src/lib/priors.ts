/**
 * Cold start priors for pattern matching.
 *
 * Uses known crash patterns from community databases to provide
 * useful matches even with N=1 crashes.
 */

export interface CrashPrior {
	id: string;
	gameId: string;
	signaturePattern: string;
	matchType: 'exact' | 'contains' | 'regex';
	patternName: string;
	knownFix: string | null;
	suspectedMods: string[];
	source: string;
	sourceUrl: string | null;
	priorConfidence: number; // 0-100
}

export interface PriorMatch {
	prior: CrashPrior;
	matchScore: number; // 0-1, how well the signature matches
	posteriorConfidence: number; // 0-100, after Bayesian update
	dataPoints: number; // How much real data we have
}

/**
 * Check if a stack trace matches a prior's signature pattern.
 */
export function matchesSignature(
	stackTrace: string,
	faultingModule: string | null,
	prior: CrashPrior,
): { matches: boolean; score: number } {
	const pattern = prior.signaturePattern;

	switch (prior.matchType) {
		case 'exact': {
			// Exact match on faulting module or full pattern
			if (faultingModule && pattern === faultingModule.toLowerCase()) {
				return { matches: true, score: 1.0 };
			}
			const matches = stackTrace.toLowerCase().includes(pattern.toLowerCase());
			return { matches, score: matches ? 0.9 : 0 };
		}

		case 'contains': {
			// Pattern must be contained in stack trace
			const lowerTrace = stackTrace.toLowerCase();
			const lowerPattern = pattern.toLowerCase();

			if (lowerTrace.includes(lowerPattern)) {
				// Score based on how early the pattern appears
				const position = lowerTrace.indexOf(lowerPattern);
				const score = Math.max(0.5, 1 - position / 1000);
				return { matches: true, score };
			}
			return { matches: false, score: 0 };
		}

		case 'regex': {
			try {
				const regex = new RegExp(pattern, 'i');
				const match = stackTrace.match(regex);
				if (match) {
					// Score based on match length relative to pattern
					const score = Math.min(1, match[0].length / pattern.length);
					return { matches: true, score: Math.max(0.5, score) };
				}
			} catch {
				// Invalid regex, treat as contains
				const matches = stackTrace
					.toLowerCase()
					.includes(pattern.toLowerCase());
				return { matches, score: matches ? 0.5 : 0 };
			}
			return { matches: false, score: 0 };
		}

		default:
			return { matches: false, score: 0 };
	}
}

/**
 * Find matching priors for a crash.
 */
export function findMatchingPriors(
	stackTrace: string,
	faultingModule: string | null,
	gameId: string,
	priors: CrashPrior[],
): PriorMatch[] {
	const matches: PriorMatch[] = [];

	for (const prior of priors) {
		if (prior.gameId !== gameId && prior.gameId !== '*') {
			continue;
		}

		const result = matchesSignature(stackTrace, faultingModule, prior);

		if (result.matches) {
			matches.push({
				prior,
				matchScore: result.score,
				posteriorConfidence: prior.priorConfidence, // Will be updated with real data
				dataPoints: 0,
			});
		}
	}

	// Sort by match score * prior confidence
	matches.sort(
		(a, b) =>
			b.matchScore * b.prior.priorConfidence -
			a.matchScore * a.prior.priorConfidence,
	);

	return matches;
}

/**
 * Bayesian update of prior confidence with real data.
 *
 * As we get more real crashes matching this pattern, the prior
 * becomes less important and real data dominates.
 */
export function updatePosterior(
	priorConfidence: number,
	realDataConfidence: number,
	dataPoints: number,
): number {
	// Prior weight decays as data accumulates
	// At 0 data points, prior dominates
	// At 10 data points, roughly equal
	// At 50+ data points, real data dominates
	const priorWeight = 1 / (1 + dataPoints / 10);

	const posterior =
		priorWeight * priorConfidence + (1 - priorWeight) * realDataConfidence;

	return Math.round(posterior);
}

/**
 * Apply Bayesian update to a prior match.
 */
export function applyBayesianUpdate(
	match: PriorMatch,
	patternOccurrenceCount: number,
	patternConfidence: number, // Confidence from real pattern data
): PriorMatch {
	return {
		...match,
		dataPoints: patternOccurrenceCount,
		posteriorConfidence: updatePosterior(
			match.prior.priorConfidence * match.matchScore,
			patternConfidence,
			patternOccurrenceCount,
		),
	};
}

/**
 * Default priors for Skyrim crashes.
 * These come from common crash patterns identified by the community.
 */
export const SKYRIM_PRIORS: Omit<CrashPrior, 'id'>[] = [
	{
		gameId: 'skyrim',
		signaturePattern: 'skyrimse.exe+12fdd00',
		matchType: 'contains',
		patternName: 'Corrupted NIF mesh',
		knownFix: 'Find and replace corrupted mesh files. Use SSE NIF Optimizer.',
		suspectedMods: [],
		source: 'community-wiki',
		sourceUrl: 'https://www.nexusmods.com/skyrimspecialedition/articles/4000',
		priorConfidence: 80,
	},
	{
		gameId: 'skyrim',
		signaturePattern: 'nvwgf2umx.dll',
		matchType: 'contains',
		patternName: 'NVIDIA driver crash',
		knownFix:
			'Update NVIDIA drivers. If using ENB, ensure compatibility with driver version.',
		suspectedMods: ['ENB'],
		source: 'community-wiki',
		sourceUrl: null,
		priorConfidence: 75,
	},
	{
		gameId: 'skyrim',
		signaturePattern: 'atiuxp64.dll',
		matchType: 'contains',
		patternName: 'AMD driver crash',
		knownFix: 'Update AMD drivers. Disable certain ENB features if needed.',
		suspectedMods: ['ENB'],
		source: 'community-wiki',
		sourceUrl: null,
		priorConfidence: 75,
	},
	{
		gameId: 'skyrim',
		signaturePattern: 'skse64_loader',
		matchType: 'contains',
		patternName: 'SKSE load failure',
		knownFix:
			'Verify SKSE version matches game version. Run as administrator.',
		suspectedMods: [],
		source: 'manual',
		sourceUrl: null,
		priorConfidence: 70,
	},
	{
		gameId: 'skyrim',
		signaturePattern: 'hkbclipgenerator',
		matchType: 'contains',
		patternName: 'Animation crash',
		knownFix:
			'Regenerate FNIS/Nemesis output. Check for conflicting animation mods.',
		suspectedMods: ['FNIS', 'Nemesis', 'DAR', 'OAR'],
		source: 'crash-log-decoder',
		sourceUrl: null,
		priorConfidence: 70,
	},
];

/**
 * Default priors for Fallout 4 crashes.
 */
export const FALLOUT4_PRIORS: Omit<CrashPrior, 'id'>[] = [
	{
		gameId: 'fallout4',
		signaturePattern: 'fallout4.exe+d5d9bc',
		matchType: 'contains',
		patternName: 'Texture loading crash',
		knownFix:
			'Reduce texture mod sizes. Install BiRaitBec texture optimization.',
		suspectedMods: [],
		source: 'community-wiki',
		sourceUrl: null,
		priorConfidence: 75,
	},
	{
		gameId: 'fallout4',
		signaturePattern: 'ba2extract',
		matchType: 'contains',
		patternName: 'BA2 archive corruption',
		knownFix: 'Verify game files. Reinstall affected mod.',
		suspectedMods: [],
		source: 'manual',
		sourceUrl: null,
		priorConfidence: 65,
	},
];

/**
 * Default priors for Cyberpunk 2077 crashes.
 */
export const CYBERPUNK_PRIORS: Omit<CrashPrior, 'id'>[] = [
	{
		gameId: 'cyberpunk',
		signaturePattern: 'cyberpunk2077.exe',
		matchType: 'contains',
		patternName: 'Generic game crash',
		knownFix:
			'Check for mod conflicts. Verify game files. Update RED4ext and CET.',
		suspectedMods: [],
		source: 'manual',
		sourceUrl: null,
		priorConfidence: 30, // Low confidence - too generic
	},
	{
		gameId: 'cyberpunk',
		signaturePattern: 'cyber_engine_tweaks',
		matchType: 'contains',
		patternName: 'CET crash',
		knownFix: 'Update Cyber Engine Tweaks. Check Lua mod errors in CET log.',
		suspectedMods: ['CET'],
		source: 'manual',
		sourceUrl: null,
		priorConfidence: 70,
	},
	{
		gameId: 'cyberpunk',
		signaturePattern: 'red4ext',
		matchType: 'contains',
		patternName: 'RED4ext plugin crash',
		knownFix:
			'Update RED4ext. Check for outdated RED4ext plugins after game updates.',
		suspectedMods: ['RED4ext'],
		source: 'manual',
		sourceUrl: null,
		priorConfidence: 70,
	},
];

/**
 * Get all default priors.
 */
export function getDefaultPriors(): Omit<CrashPrior, 'id'>[] {
	return [...SKYRIM_PRIORS, ...FALLOUT4_PRIORS, ...CYBERPUNK_PRIORS];
}
