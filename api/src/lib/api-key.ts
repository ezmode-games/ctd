import { createHash, randomBytes } from 'crypto';

const CHARSET =
	'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';

/**
 * Generate a new API key with cryptographically secure randomness.
 * Format: ctd_<32 alphanumeric chars>
 * Example: ctd_a8Kj2mNp4qRs6tUv8wXy0zB3dF5gH7jL
 */
export function generateApiKey(): string {
	const bytes = randomBytes(32);
	let key = 'ctd_';
	for (let i = 0; i < 32; i++) {
		key += CHARSET[bytes[i] % CHARSET.length];
	}
	return key;
}

/**
 * Hash an API key using SHA-256 for secure storage.
 */
export function hashApiKey(key: string): string {
	return createHash('sha256').update(key).digest('hex');
}

/**
 * Get the prefix of an API key for identification.
 * Returns the first 8 characters (e.g., "ctd_xxxx").
 */
export function getKeyPrefix(key: string): string {
	return key.slice(0, 8);
}

/**
 * Validate that a string matches the API key format.
 */
export function isValidApiKeyFormat(key: string): boolean {
	return /^ctd_[a-zA-Z0-9]{32}$/.test(key);
}
