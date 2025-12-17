import { OpenAPIHono } from '@hono/zod-openapi';
import { requestId } from 'hono/request-id';
import { notFound, onError } from 'stoker/middlewares';
import { defaultHook } from 'stoker/openapi';

export function createRouter() {
	return new OpenAPIHono({
		strict: false,
		defaultHook,
	});
}

export default function createApp() {
	const app = createRouter();
	app.use(requestId());
	app.notFound(notFound);
	app.onError(onError);
	return app;
}

export function createTestApp(router: OpenAPIHono) {
	return createApp().route('/', router);
}
