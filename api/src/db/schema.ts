import { sql } from 'drizzle-orm';
import { integer, sqliteTable, text } from 'drizzle-orm/sqlite-core';

export const crashReport = sqliteTable('crash_report', {
	id: text('id').primaryKey(), // ULID
	schemaVersion: integer('schema_version').notNull().default(1),
	gameId: text('game_id').notNull(),
	userId: text('user_id'),

	// Crash identification
	crashHash: text('crash_hash').notNull(),
	stackTrace: text('stack_trace').notNull(),
	exceptionCode: text('exception_code'),
	exceptionAddress: text('exception_address'),
	faultingModule: text('faulting_module'),

	// Environment
	gameVersion: text('game_version').notNull(),
	scriptExtenderVersion: text('script_extender_version'),
	osVersion: text('os_version'),

	// Load order (JSON array)
	loadOrderJson: text('load_order_json').notNull(),
	pluginCount: integer('plugin_count').notNull(),

	// Timestamps (stored as ms since epoch)
	crashedAt: integer('crashed_at', { mode: 'timestamp_ms' }).notNull(),
	submittedAt: integer('submitted_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`),

	// Sharing
	isPublic: integer('is_public', { mode: 'boolean' }).notNull().default(false),
	shareToken: text('share_token').unique(),

	notes: text('notes'),
	createdAt: integer('created_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`),
});

export const apiKey = sqliteTable('api_key', {
	id: text('id').primaryKey(), // ULID
	userId: text('user_id'), // Optional - for future user association
	name: text('name').notNull().default('default'),
	keyHash: text('key_hash').notNull().unique(), // SHA-256 hash of key
	keyPrefix: text('key_prefix').notNull(), // First 8 chars for identification (ctd_xxxx)
	lastUsedAt: integer('last_used_at', { mode: 'timestamp_ms' }),
	expiresAt: integer('expires_at', { mode: 'timestamp_ms' }),
	createdAt: integer('created_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`),
});

export const crashPattern = sqliteTable('crash_pattern', {
	id: text('id').primaryKey(), // ULID
	gameId: text('game_id').notNull(),
	crashHash: text('crash_hash').notNull().unique(),
	patternName: text('pattern_name'),
	occurrenceCount: integer('occurrence_count').notNull().default(1),
	firstSeenAt: integer('first_seen_at', { mode: 'timestamp_ms' }).notNull(),
	lastSeenAt: integer('last_seen_at', { mode: 'timestamp_ms' }).notNull(),
	suspectedModsJson: text('suspected_mods_json'),
	knownFix: text('known_fix'),
	isResolved: integer('is_resolved', { mode: 'boolean' })
		.notNull()
		.default(false),
	createdAt: integer('created_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`),
	updatedAt: integer('updated_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`),
});

// Mod correlation with crash patterns (#62)
export const patternModCorrelation = sqliteTable('pattern_mod_correlation', {
	id: text('id').primaryKey(), // ULID
	patternId: text('pattern_id')
		.notNull()
		.references(() => crashPattern.id),
	gameId: text('game_id').notNull(),

	// Mod identification (name or fingerprint)
	modName: text('mod_name').notNull(),
	modFingerprint: text('mod_fingerprint'), // xxh3 hash if available

	// Raw counts
	crashesWithMod: integer('crashes_with_mod').notNull().default(0),
	crashesWithoutMod: integer('crashes_without_mod').notNull().default(0),
	totalCrashes: integer('total_crashes').notNull().default(0),

	// Computed metrics
	lift: integer('lift').notNull().default(100), // Stored as percentage (100 = 1.0)
	correlationScore: integer('correlation_score').notNull().default(0), // 0-100
	confidence: integer('confidence').notNull().default(0), // 0-100

	updatedAt: integer('updated_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`),
});

// Known crash patterns from community (#63)
export const crashPrior = sqliteTable('crash_prior', {
	id: text('id').primaryKey(), // ULID
	gameId: text('game_id').notNull(),

	// Pattern matching
	signaturePattern: text('signature_pattern').notNull(), // Regex or module+offset
	matchType: text('match_type').notNull().default('contains'), // 'exact', 'contains', 'regex'

	// Knowledge
	patternName: text('pattern_name').notNull(),
	knownFix: text('known_fix'),
	suspectedModsJson: text('suspected_mods_json'), // JSON array of mod names

	// Provenance
	source: text('source').notNull(), // 'crash-log-decoder', 'community-wiki', 'manual'
	sourceUrl: text('source_url'),
	priorConfidence: integer('prior_confidence').notNull().default(70), // 0-100

	createdAt: integer('created_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`),
	updatedAt: integer('updated_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`),
});

// Prediction tracking for calibration (#64)
export const patternPrediction = sqliteTable('pattern_prediction', {
	id: text('id').primaryKey(), // ULID
	crashId: text('crash_id')
		.notNull()
		.references(() => crashReport.id),
	patternId: text('pattern_id').references(() => crashPattern.id),
	priorId: text('prior_id').references(() => crashPrior.id),

	// Prediction details
	predictedConfidence: integer('predicted_confidence').notNull(), // 0-100
	calibratedConfidence: integer('calibrated_confidence'), // After recalibration
	predictionMethod: text('prediction_method').notNull(), // 'hash_match', 'prior', 'similarity'

	// Validation
	wasCorrect: integer('was_correct', { mode: 'boolean' }),
	validatedAt: integer('validated_at', { mode: 'timestamp_ms' }),
	validationSource: text('validation_source'), // 'user_feedback', 'mod_author', 'auto'

	createdAt: integer('created_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`),
});

// Frame embeddings for similarity search (#61)
export const crashEmbedding = sqliteTable('crash_embedding', {
	id: text('id').primaryKey(), // Same as crash_report.id
	crashId: text('crash_id')
		.notNull()
		.unique()
		.references(() => crashReport.id),
	gameId: text('game_id').notNull(),

	// Vector stored as JSON array of floats
	vectorJson: text('vector_json').notNull(),
	vectorDim: integer('vector_dim').notNull(),

	// Metadata for filtering
	faultingModule: text('faulting_module'),

	createdAt: integer('created_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`),
});

// Game-wide mod statistics for base rates
export const gameModStats = sqliteTable('game_mod_stats', {
	id: text('id').primaryKey(), // ULID
	gameId: text('game_id').notNull(),
	modName: text('mod_name').notNull(),
	modFingerprint: text('mod_fingerprint'),

	// Usage stats
	seenInCrashes: integer('seen_in_crashes').notNull().default(0),
	totalCrashesForGame: integer('total_crashes_for_game').notNull().default(0),
	baseRatePct: integer('base_rate_pct').notNull().default(0), // 0-10000 (2 decimal places)

	updatedAt: integer('updated_at', { mode: 'timestamp_ms' })
		.notNull()
		.default(sql`(unixepoch() * 1000)`),
});

export type CrashReport = typeof crashReport.$inferSelect;
export type NewCrashReport = typeof crashReport.$inferInsert;
export type CrashPattern = typeof crashPattern.$inferSelect;
export type NewCrashPattern = typeof crashPattern.$inferInsert;
export type ApiKey = typeof apiKey.$inferSelect;
export type NewApiKey = typeof apiKey.$inferInsert;
export type PatternModCorrelation = typeof patternModCorrelation.$inferSelect;
export type CrashPrior = typeof crashPrior.$inferSelect;
export type PatternPrediction = typeof patternPrediction.$inferSelect;
export type CrashEmbedding = typeof crashEmbedding.$inferSelect;
export type GameModStats = typeof gameModStats.$inferSelect;
