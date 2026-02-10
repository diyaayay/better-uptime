-- Your SQL goes here
-- Add monitoring fields to website table
ALTER TABLE website 
ADD COLUMN is_up BOOLEAN DEFAULT true,
ADD COLUMN last_checked TIMESTAMP,
ADD COLUMN last_down_time TIMESTAMP,
ADD COLUMN response_time_ms INTEGER;
-- Create check_history table to track all checks
CREATE TABLE check_history (
    id VARCHAR(255) PRIMARY KEY,
    website_id VARCHAR(255) NOT NULL REFERENCES website(id) ON DELETE CASCADE,
    checked_at TIMESTAMP NOT NULL,
    is_up BOOLEAN NOT NULL,
    response_time_ms INTEGER,
    status_code INTEGER,
    error_message TEXT
);

CREATE INDEX idx_check_history_website_id ON check_history(website_id);
CREATE INDEX idx_check_history_checked_at ON check_history(checked_at);