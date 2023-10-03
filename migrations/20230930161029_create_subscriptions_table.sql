-- Add migration script here
CREATE TABLE subscriptions(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  email TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  subscribed_at timestamptz NOT NULL
);

INSERT INTO subscriptions
VALUES(gen_random_uuid(), 'test@test.com', 'test', '2023-09-30T14:05:00.000Z');