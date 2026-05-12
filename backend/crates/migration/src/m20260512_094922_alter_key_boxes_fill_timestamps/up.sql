UPDATE key_boxes
SET expires_at = now() + INTERVAL '60 days'
WHERE expires_at IS NULL;

UPDATE key_boxes
SET activated_at = now()
WHERE activated_at IS NULL;

UPDATE key_boxes
SET retired_at = LEAST(
    now() + INTERVAL '30 days',
    expires_at - INTERVAL '1 second'
)
WHERE retired_at IS NULL;

ALTER TABLE key_boxes
    ALTER COLUMN activated_at SET NOT NULL,
    ALTER COLUMN retired_at SET NOT NULL,
    ALTER COLUMN expires_at SET NOT NULL;
