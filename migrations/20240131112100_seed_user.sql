-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
  '2d8c861e-b2e6-4a00-bb44-7dfe4cd3bc3b',
  'admin',
  '$argon2id$v=19$m=15000,t=2,p=1$ZegXdWjxHc6jQE+2L+CtDw$84vf+M9sDkwsWLMegUsdwezd9qrt9OeGT1/ybKfxIYY'
);