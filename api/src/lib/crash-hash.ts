import { createHash } from 'node:crypto';

export interface StackFrame {
	module: string;
	offset: string;
	isSystemFrame: boolean;
}

const SYSTEM_MODULES = new Set([
	'ntdll.dll',
	'kernel32.dll',
	'kernelbase.dll',
	'win32u.dll',
	'user32.dll',
	'gdi32.dll',
	'msvcrt.dll',
	'ucrtbase.dll',
	'vcruntime140.dll',
	'msvcp140.dll',
]);

export function isSystemModule(module: string): boolean {
	return SYSTEM_MODULES.has(module.toLowerCase());
}

// Crash Logger format: [0] 0x7FF712345678 SkyrimSE.exe+0x12345
const CRASH_LOGGER_REGEX = /\[\d+\]\s+0x[0-9A-Fa-f]+\s+([^\s+]+)\+(\S+)/;

// .NET Script Framework format: SkyrimSE.exe+12345
const SCRIPT_FRAMEWORK_REGEX = /^([^\s+]+)\+(\S+)$/;

export function parseStackTrace(stackTrace: string): StackFrame[] {
	if (!stackTrace.trim()) {
		return [];
	}

	const frames: StackFrame[] = [];
	const lines = stackTrace.split('\n');

	for (const line of lines) {
		const trimmed = line.trim();
		if (!trimmed) continue;

		let match = trimmed.match(CRASH_LOGGER_REGEX);
		if (match) {
			const module = match[1];
			frames.push({
				module,
				offset: match[2],
				isSystemFrame: isSystemModule(module),
			});
			continue;
		}

		match = trimmed.match(SCRIPT_FRAMEWORK_REGEX);
		if (match) {
			const module = match[1];
			frames.push({
				module,
				offset: match[2],
				isSystemFrame: isSystemModule(module),
			});
		}
	}

	return frames;
}

function sha256(input: string): string {
	return createHash('sha256').update(input).digest('hex');
}

export function computeCrashHash(stackTrace: string): string {
	const frames = parseStackTrace(stackTrace);

	const normalized = frames
		.filter((f) => !f.isSystemFrame)
		.slice(0, 10)
		.map((f) => `${f.module.toLowerCase()}+${f.offset}`)
		.join('|');

	if (!normalized) {
		// Fallback: hash entire trace if no game frames
		return sha256(stackTrace).slice(0, 16);
	}

	return sha256(normalized).slice(0, 16);
}
