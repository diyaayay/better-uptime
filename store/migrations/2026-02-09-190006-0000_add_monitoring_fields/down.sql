-- This file should undo anything in `up.sql`
ALTER TABLE website 
DROP COLUMN is_up,
DROP COLUMN last_checked,
DROP COLUMN last_down_time,
DROP COLUMN response_time_ms;

DROP TABLE IF EXISTS check_history;