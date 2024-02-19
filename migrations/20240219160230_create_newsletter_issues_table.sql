-- Add migration script here
CREATE TABLE newsletter_issues (
  newsletter_issue_id uuid NOT NULL,
  title TEXT NOT NULL,
  text_context TEXT NOT NULL,
  html_content TEXT NOT NULL,
  published_at TEXT NOT NULL,
  PRIMARY KEY(newsletter_issue_id)
);