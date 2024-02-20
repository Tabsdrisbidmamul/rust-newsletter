-- Add migration script here
ALTER TABLE newsletter_issues RENAME COLUMN text_context TO text_content;