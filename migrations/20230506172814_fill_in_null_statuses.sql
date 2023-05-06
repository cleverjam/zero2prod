-- Add migration script here
-- Transaction:
BEGIN;
-- fill in empty statuses
UPDATE subscriptions
SET status = 'confirmed'
WHERE status is NULL;
-- make status column required
ALTER TABLE subscriptions
    ALTER COLUMN status SET NOT NULL;
COMMIT;


