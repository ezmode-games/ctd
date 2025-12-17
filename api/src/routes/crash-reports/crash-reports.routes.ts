import { createRoute, z } from '@hono/zod-openapi';
import * as HttpStatusCodes from 'stoker/http-status-codes';
import { jsonContent } from 'stoker/openapi/helpers';

import {
	crashReportResponseSchema,
	createCrashReportSchema,
} from '@/db/schema.zod';

const errorSchema = z.object({
	error: z.object({
		code: z.string(),
		message: z.string(),
	}),
});

export const submitCrashReport = createRoute({
	method: 'post',
	path: '/api/crash-reports',
	tags: ['Crash Reports'],
	summary: 'Submit a new crash report',
	request: {
		body: {
			content: {
				'application/json': {
					schema: createCrashReportSchema,
				},
			},
		},
	},
	responses: {
		[HttpStatusCodes.CREATED]: jsonContent(
			crashReportResponseSchema,
			'Crash report created',
		),
		[HttpStatusCodes.UNPROCESSABLE_ENTITY]: jsonContent(
			errorSchema,
			'Validation error',
		),
	},
});

export type SubmitCrashReportRoute = typeof submitCrashReport;
