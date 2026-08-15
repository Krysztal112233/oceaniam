ALTER TABLE subjects DROP COLUMN expires_at;

-- Guarded so a partially-applied `up` (extension created, queue creation failed) does not
-- make `down` fail: `pgmq.drop_queue` raises when the queue does not exist.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pgmq.meta WHERE queue_name = 'dev_account_expiration') THEN
        PERFORM pgmq.drop_queue('dev_account_expiration');
    END IF;
END $$;
