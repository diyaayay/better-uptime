-- This file should undo anything in `up.sql`
ALTER TABLE "user"
DROP CONSTRAINT user_username_unique;