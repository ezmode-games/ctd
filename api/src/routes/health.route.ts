import { createRoute, z } from '@hono/zod-openapi';
import * as HttpStatusCodes from 'stoker/http-status-codes';
import { jsonContent } from 'stoker/openapi/helpers';

import { createRouter } from '@/lib/create-app';

const healthResponseSchema = z.object({
	status: z.literal('ok'),
	version: z.string(),
});

const router = createRouter().openapi(
	createRoute({
		tags: ['Health'],
		method: 'get',
		path: '/health',
		responses: {
			[HttpStatusCodes.OK]: jsonContent(healthResponseSchema, 'Health check'),
		},
	}),
	(c) => {
		return c.json(
			{
				status: 'ok' as const,
				version: '0.1.0',
			},
			HttpStatusCodes.OK,
		);
	},
);

export default router;
