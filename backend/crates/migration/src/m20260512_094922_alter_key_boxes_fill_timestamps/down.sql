ALTER TABLE key_boxes
    ALTER COLUMN activated_at DROP NOT NULL,
    ALTER COLUMN retired_at DROP NOT NULL,
    ALTER COLUMN expires_at DROP NOT NULL;
