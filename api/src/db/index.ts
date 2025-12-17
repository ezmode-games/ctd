/**
 * Database connection setup.
 *
 * For self-hosting, configure this file with your database connection.
 * By default, uses SQLite in the data/ directory.
 *
 * Example configurations:
 *
 * SQLite (default):
 *   import Database from 'better-sqlite3';
 *   import { drizzle } from 'drizzle-orm/better-sqlite3';
 *   const sqlite = new Database('./data/ctd.db');
 *   export const db = drizzle(sqlite, { schema });
 *
 * PostgreSQL:
 *   import { drizzle } from 'drizzle-orm/node-postgres';
 *   import { Pool } from 'pg';
 *   const pool = new Pool({ connectionString: process.env.DATABASE_URL });
 *   export const db = drizzle(pool, { schema });
 */

import Database from 'better-sqlite3';
import { drizzle } from 'drizzle-orm/better-sqlite3';
import * as schema from './schema.js';

// Ensure data directory exists
import { mkdirSync } from 'node:fs';
import { dirname } from 'node:path';

const dbPath = './data/ctd.db';
mkdirSync(dirname(dbPath), { recursive: true });

const sqlite = new Database(dbPath);
sqlite.pragma('journal_mode = WAL');

export const db = drizzle(sqlite, { schema });

export * from './schema.js';
