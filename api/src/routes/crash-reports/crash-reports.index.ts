import { createRouter } from '@/lib/create-app';

import * as handlers from './crash-reports.handlers';
import * as routes from './crash-reports.routes';

const router = createRouter().openapi(
	routes.submitCrashReport,
	handlers.submitCrashReport,
);

export default router;
