import { serve } from '@hono/node-server';

import { app } from './app';

const port = Number(process.env.PORT) || 3000;

console.log(`Starting CTD API on port ${port}`);

serve({
	fetch: app.fetch,
	port,
});

console.log(`CTD API running at http://localhost:${port}`);
