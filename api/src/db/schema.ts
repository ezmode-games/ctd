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

export type CrashReport = typeof crashReport.$inferSelect;
export type NewCrashReport = typeof crashReport.$inferInsert;
export type CrashPattern = typeof crashPattern.$inferSelect;
export type NewCrashPattern = typeof crashPattern.$inferInsert;
export type ApiKey = typeof apiKey.$inferSelect;
export type NewApiKey = typeof apiKey.$inferInsert;
