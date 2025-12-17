import { z } from 'zod';

export const loadOrderItemSchema = z.object({
	name: z.string(),
	enabled: z.boolean().optional(),
	index: z.number().int().optional(),
});

export const loadOrderSchema = z.array(loadOrderItemSchema);

// Individual field schemas for zocker supply()
export const gameIdSchema = z.string().min(1);
export const stackTraceSchema = z.string().min(1).max(100000);
export const gameVersionSchema = z.string().min(1).max(50);
export const loadOrderJsonSchema = z.string().refine(
	(val) => {
		try {
			const parsed = JSON.parse(val);
			return Array.isArray(parsed);
		} catch {
			return false;
		}
	},
	{ message: 'loadOrderJson must be a valid JSON array' },
);
export const pluginCountSchema = z.number().int().min(0).max(10000);
export const crashedAtSchema = z.number().int().positive();

export const createCrashReportSchema = z.object({
	schemaVersion: z.number().int().min(1).default(1),
	gameId: gameIdSchema,
	stackTrace: stackTraceSchema,
	crashHash: z.string().min(1).max(64).optional(),
	exceptionCode: z.string().max(50).optional(),
	exceptionAddress: z.string().max(50).optional(),
	faultingModule: z.string().max(255).optional(),
	gameVersion: gameVersionSchema,
	scriptExtenderVersion: z.string().max(50).optional(),
	osVersion: z.string().max(100).optional(),
	loadOrderJson: loadOrderJsonSchema,
	pluginCount: pluginCountSchema,
	crashedAt: crashedAtSchema,
	notes: z.string().max(5000).optional(),
});

export const crashReportResponseSchema = z.object({
	id: z.string(),
	shareToken: z.string(),
});

export type CreateCrashReport = z.infer<typeof createCrashReportSchema>;
export type CrashReportResponse = z.infer<typeof crashReportResponseSchema>;
