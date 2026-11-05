ALTER TABLE key_boxes
ADD COLUMN IF NOT EXISTS kek_version INTEGER;

-- Keep the JSON field as a compatibility mirror for older binaries during rolling deployment.
-- New code treats this column as authoritative and validates the mirror.
UPDATE key_boxes
SET kek_version = (secret->>'key_version')::integer
WHERE kek_version IS NULL;

ALTER TABLE key_boxes
ALTER COLUMN kek_version SET NOT NULL;

DO $$ BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ck_key_boxes_kek_version_positive'
        AND conrelid = 'key_boxes'::regclass
    ) THEN
        ALTER TABLE key_boxes
        ADD CONSTRAINT ck_key_boxes_kek_version_positive
        CHECK (kek_version > 0);
    END IF;
END $$;
