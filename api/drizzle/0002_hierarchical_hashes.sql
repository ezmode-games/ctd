-- Add hierarchical hash columns for confidence-based pattern matching
-- L1 (broadest) to L4, with existing crash_hash serving as L5 (most specific)
--
-- See: https://github.com/ezmode-games/ctd/issues/60

ALTER TABLE crash_report ADD COLUMN l1_hash TEXT;
--> statement-breakpoint
ALTER TABLE crash_report ADD COLUMN l2_hash TEXT;
--> statement-breakpoint
ALTER TABLE crash_report ADD COLUMN l3_hash TEXT;
--> statement-breakpoint
ALTER TABLE crash_report ADD COLUMN l4_hash TEXT;
--> statement-breakpoint
CREATE INDEX idx_crash_report_l1_hash ON crash_report(l1_hash);
--> statement-breakpoint
CREATE INDEX idx_crash_report_l2_hash ON crash_report(l2_hash);
--> statement-breakpoint
CREATE INDEX idx_crash_report_l3_hash ON crash_report(l3_hash);
--> statement-breakpoint
CREATE INDEX idx_crash_report_l4_hash ON crash_report(l4_hash);
