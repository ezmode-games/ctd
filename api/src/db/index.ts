/**
 * Database connection setup.
 *
 * For self-hosting, configure this file with your database connection.
 * By default, uses SQLite in the data/ directory.
 *
 * Set DATABASE_URL=:memory: for in-memory database (used in tests).
 */

import { mkdirSync } from 'node:fs';
import { dirname } from 'node:path';
import Database from 'better-sqlite3';
import { drizzle } from 'drizzle-orm/better-sqlite3';
import * as schema from './schema.js';

const dbPath = process.env.DATABASE_URL || './data/ctd.db';
const isMemory = dbPath === ':memory:';

if (!isMemory) {
	mkdirSync(dirname(dbPath), { recursive: true });
}

const sqlite = new Database(dbPath);

if (!isMemory) {
	sqlite.pragma('journal_mode = WAL');
}

// Create tables for in-memory database
if (isMemory) {
	sqlite.exec(`
		CREATE TABLE IF NOT EXISTS crash_report (
			id TEXT PRIMARY KEY,
			game_id TEXT NOT NULL,
			user_id TEXT,
			crash_hash TEXT NOT NULL,
			stack_trace TEXT NOT NULL,
			exception_code TEXT,
			exception_address TEXT,
			faulting_module TEXT,
			game_version TEXT NOT NULL,
			script_extender_version TEXT,
			os_version TEXT,
			load_order_json TEXT NOT NULL,
			plugin_count INTEGER NOT NULL,
			crashed_at INTEGER NOT NULL,
			submitted_at INTEGER NOT NULL DEFAULT (unixepoch() * 1000),
			is_public INTEGER NOT NULL DEFAULT 0,
			share_token TEXT UNIQUE,
			notes TEXT,
			created_at INTEGER NOT NULL DEFAULT (unixepoch() * 1000)
		);

		CREATE TABLE IF NOT EXISTS crash_pattern (
			id TEXT PRIMARY KEY,
			game_id TEXT NOT NULL,
			crash_hash TEXT NOT NULL UNIQUE,
			pattern_name TEXT,
			occurrence_count INTEGER NOT NULL DEFAULT 1,
			first_seen_at INTEGER NOT NULL,
			last_seen_at INTEGER NOT NULL,
			suspected_mods_json TEXT,
			known_fix TEXT,
			is_resolved INTEGER NOT NULL DEFAULT 0,
			created_at INTEGER NOT NULL DEFAULT (unixepoch() * 1000),
			updated_at INTEGER NOT NULL DEFAULT (unixepoch() * 1000)
		);
	`);
}

export const db = drizzle(sqlite, { schema });

export * from './schema.js';
