-- Add migration script here
BEGIN TRANSACTION;
  -- backfill `status` to keep historical data
  UPDATE subscriptions 
    SET status = 'confirmed' 
    WHERE status IS NULL;
  
  -- make status mandatory of string type 
  ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT TRANSACTION;