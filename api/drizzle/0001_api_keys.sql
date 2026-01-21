CREATE TABLE `api_key` (
	`id` text PRIMARY KEY NOT NULL,
	`user_id` text,
	`name` text DEFAULT 'default' NOT NULL,
	`key_hash` text NOT NULL,
	`key_prefix` text NOT NULL,
	`last_used_at` integer,
	`expires_at` integer,
	`created_at` integer DEFAULT (unixepoch() * 1000) NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `api_key_key_hash_unique` ON `api_key` (`key_hash`);
