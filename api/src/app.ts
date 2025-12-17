import configureOpenAPI from '@/lib/configure-open-api';
import createApp from '@/lib/create-app';
import crashReports from '@/routes/crash-reports/crash-reports.index';
import health from '@/routes/health.route';
import index from '@/routes/index.route';

const app = createApp();

configureOpenAPI(app);

const routes = [index, health, crashReports] as const;

for (const route of routes) {
	app.route('/', route);
}

export type AppType = (typeof routes)[number];

export { app };
