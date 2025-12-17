import { mkdirSync } from 'node:fs';
import { dirname } from 'node:path';
import Database from 'better-sqlite3';
import { drizzle } from 'drizzle-orm/better-sqlite3';
import { migrate } from 'drizzle-orm/better-sqlite3/migrator';
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

export const db = drizzle(sqlite, { schema });

// Run migrations (works for both file and in-memory databases)
const migrationsFolder = new URL('../../drizzle', import.meta.url).pathname;
migrate(db, { migrationsFolder });

export * from './schema.js';
export * from './schema.zod.js';
