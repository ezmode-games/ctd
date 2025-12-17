import { zocker } from 'zocker';

import {
	crashedAtSchema,
	createCrashReportSchema,
	gameIdSchema,
	gameVersionSchema,
	loadOrderItemSchema,
	loadOrderJsonSchema,
	pluginCountSchema,
	stackTraceSchema,
} from '@/db/schema.zod';

// Create a configured zocker instance for crash reports with realistic test data
function createCrashReportZocker() {
	return zocker(createCrashReportSchema)
		.supply(gameIdSchema, 'skyrim-se')
		.supply(gameVersionSchema, '1.6.1130')
		.supply(stackTraceSchema, 'SkyrimSE.exe+0x12345\nENBSeries.dll+0x67890')
		.supply(
			loadOrderJsonSchema,
			JSON.stringify([
				{ name: 'Skyrim.esm', enabled: true, index: 0 },
				{ name: 'Update.esm', enabled: true, index: 1 },
				{ name: 'SkyUI_SE.esp', enabled: true, index: 2 },
			]),
		)
		.supply(pluginCountSchema, 3)
		.supply(crashedAtSchema, Date.now());
}

// Generate mock load order items
export function mockLoadOrderItem() {
	return zocker(loadOrderItemSchema).generate();
}

// Generate mock crash reports
export function mockCrashReport() {
	return createCrashReportZocker().generate();
}

// Generate a batch of crash reports
export function mockCrashReportBatch(count: number) {
	return createCrashReportZocker().generateMany(count);
}
