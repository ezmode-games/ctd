CREATE TABLE `crash_pattern` (
	`id` text PRIMARY KEY NOT NULL,
	`game_id` text NOT NULL,
	`crash_hash` text NOT NULL,
	`pattern_name` text,
	`occurrence_count` integer DEFAULT 1 NOT NULL,
	`first_seen_at` integer NOT NULL,
	`last_seen_at` integer NOT NULL,
	`suspected_mods_json` text,
	`known_fix` text,
	`is_resolved` integer DEFAULT false NOT NULL,
	`created_at` integer DEFAULT (unixepoch() * 1000) NOT NULL,
	`updated_at` integer DEFAULT (unixepoch() * 1000) NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `crash_pattern_crash_hash_unique` ON `crash_pattern` (`crash_hash`);--> statement-breakpoint
CREATE TABLE `crash_report` (
	`id` text PRIMARY KEY NOT NULL,
	`schema_version` integer DEFAULT 1 NOT NULL,
	`game_id` text NOT NULL,
	`user_id` text,
	`crash_hash` text NOT NULL,
	`stack_trace` text NOT NULL,
	`exception_code` text,
	`exception_address` text,
	`faulting_module` text,
	`game_version` text NOT NULL,
	`script_extender_version` text,
	`os_version` text,
	`load_order_json` text NOT NULL,
	`plugin_count` integer NOT NULL,
	`crashed_at` integer NOT NULL,
	`submitted_at` integer DEFAULT (unixepoch() * 1000) NOT NULL,
	`is_public` integer DEFAULT false NOT NULL,
	`share_token` text,
	`notes` text,
	`created_at` integer DEFAULT (unixepoch() * 1000) NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `crash_report_share_token_unique` ON `crash_report` (`share_token`);