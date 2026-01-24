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
CREATE UNIQUE INDEX `api_key_key_hash_unique` ON `api_key` (`key_hash`);--> statement-breakpoint
CREATE TABLE `crash_embedding` (
	`id` text PRIMARY KEY NOT NULL,
	`crash_id` text NOT NULL,
	`game_id` text NOT NULL,
	`vector_json` text NOT NULL,
	`vector_dim` integer NOT NULL,
	`faulting_module` text,
	`created_at` integer DEFAULT (unixepoch() * 1000) NOT NULL,
	FOREIGN KEY (`crash_id`) REFERENCES `crash_report`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE UNIQUE INDEX `crash_embedding_crash_id_unique` ON `crash_embedding` (`crash_id`);--> statement-breakpoint
CREATE TABLE `crash_prior` (
	`id` text PRIMARY KEY NOT NULL,
	`game_id` text NOT NULL,
	`signature_pattern` text NOT NULL,
	`match_type` text DEFAULT 'contains' NOT NULL,
	`pattern_name` text NOT NULL,
	`known_fix` text,
	`suspected_mods_json` text,
	`source` text NOT NULL,
	`source_url` text,
	`prior_confidence` integer DEFAULT 70 NOT NULL,
	`created_at` integer DEFAULT (unixepoch() * 1000) NOT NULL,
	`updated_at` integer DEFAULT (unixepoch() * 1000) NOT NULL
);
--> statement-breakpoint
CREATE TABLE `game_mod_stats` (
	`id` text PRIMARY KEY NOT NULL,
	`game_id` text NOT NULL,
	`mod_name` text NOT NULL,
	`mod_fingerprint` text,
	`seen_in_crashes` integer DEFAULT 0 NOT NULL,
	`total_crashes_for_game` integer DEFAULT 0 NOT NULL,
	`base_rate_pct` integer DEFAULT 0 NOT NULL,
	`updated_at` integer DEFAULT (unixepoch() * 1000) NOT NULL
);
--> statement-breakpoint
CREATE TABLE `pattern_mod_correlation` (
	`id` text PRIMARY KEY NOT NULL,
	`pattern_id` text NOT NULL,
	`game_id` text NOT NULL,
	`mod_name` text NOT NULL,
	`mod_fingerprint` text,
	`crashes_with_mod` integer DEFAULT 0 NOT NULL,
	`crashes_without_mod` integer DEFAULT 0 NOT NULL,
	`total_crashes` integer DEFAULT 0 NOT NULL,
	`lift` integer DEFAULT 100 NOT NULL,
	`correlation_score` integer DEFAULT 0 NOT NULL,
	`confidence` integer DEFAULT 0 NOT NULL,
	`updated_at` integer DEFAULT (unixepoch() * 1000) NOT NULL,
	FOREIGN KEY (`pattern_id`) REFERENCES `crash_pattern`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `pattern_prediction` (
	`id` text PRIMARY KEY NOT NULL,
	`crash_id` text NOT NULL,
	`pattern_id` text,
	`prior_id` text,
	`predicted_confidence` integer NOT NULL,
	`calibrated_confidence` integer,
	`prediction_method` text NOT NULL,
	`was_correct` integer,
	`validated_at` integer,
	`validation_source` text,
	`created_at` integer DEFAULT (unixepoch() * 1000) NOT NULL,
	FOREIGN KEY (`crash_id`) REFERENCES `crash_report`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`pattern_id`) REFERENCES `crash_pattern`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`prior_id`) REFERENCES `crash_prior`(`id`) ON UPDATE no action ON DELETE no action
);
