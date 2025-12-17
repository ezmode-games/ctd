import { Scalar } from '@scalar/hono-api-reference';

import packageJSON from '../../package.json' with { type: 'json' };
import type { AppOpenAPI } from './types';

export default function configureOpenAPI(app: AppOpenAPI) {
	app.doc('/docs', {
		openapi: '3.1.0',
		info: {
			version: packageJSON.version,
			title: 'CTD API',
			description: 'Crash To Desktop Reporter API',
			license: {
				name: 'AGPL-3.0',
				url: 'https://www.gnu.org/licenses/agpl-3.0.html',
			},
		},
		servers: [
			{
				url: 'https://ctd.ezmode.games',
				description: 'Production',
			},
			{
				url: 'http://localhost:3000',
				description: 'Local development',
			},
		],
	});

	app.get(
		'/reference',
		Scalar({
			url: '/docs',
			defaultHttpClient: {
				targetKey: 'js',
				clientKey: 'fetch',
			},
		}),
	);
}
