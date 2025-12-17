import { z } from '@hono/zod-openapi';

export const loadOrderItemSchema = z.object({
	name: z.string(),
	enabled: z.boolean().optional(),
	index: z.number().int().optional(),
});

export const loadOrderSchema = z.array(loadOrderItemSchema);

export const createCrashReportSchema = z.object({
	gameId: z.string().min(1),
	stackTrace: z.string().min(1).max(100000),
	crashHash: z.string().min(1).max(64).optional(),
	exceptionCode: z.string().max(50).optional(),
	exceptionAddress: z.string().max(50).optional(),
	faultingModule: z.string().max(255).optional(),
	gameVersion: z.string().min(1).max(50),
	scriptExtenderVersion: z.string().max(50).optional(),
	osVersion: z.string().max(100).optional(),
	loadOrderJson: z.string().refine(
		(val) => {
			try {
				const parsed = JSON.parse(val);
				return Array.isArray(parsed);
			} catch {
				return false;
			}
		},
		{ message: 'loadOrderJson must be a valid JSON array' },
	),
	pluginCount: z.number().int().min(0).max(10000),
	crashedAt: z.number().int().positive(),
	notes: z.string().max(5000).optional(),
});

export const crashReportResponseSchema = z.object({
	id: z.string(),
	shareToken: z.string(),
});

export const selectCrashReportSchema = z.object({
	id: z.string(),
	gameId: z.string(),
	crashHash: z.string(),
	stackTrace: z.string(),
	exceptionCode: z.string().nullable(),
	exceptionAddress: z.string().nullable(),
	faultingModule: z.string().nullable(),
	gameVersion: z.string(),
	scriptExtenderVersion: z.string().nullable(),
	osVersion: z.string().nullable(),
	loadOrder: loadOrderSchema,
	pluginCount: z.number(),
	crashedAt: z.number(),
	submittedAt: z.number(),
	isPublic: z.boolean(),
	notes: z.string().nullable(),
	pattern: z
		.object({
			id: z.string(),
			patternName: z.string().nullable(),
			occurrenceCount: z.number(),
			knownFix: z.string().nullable(),
		})
		.nullable(),
});

export type CreateCrashReport = z.infer<typeof createCrashReportSchema>;
export type CrashReportResponse = z.infer<typeof crashReportResponseSchema>;
export type SelectCrashReport = z.infer<typeof selectCrashReportSchema>;
