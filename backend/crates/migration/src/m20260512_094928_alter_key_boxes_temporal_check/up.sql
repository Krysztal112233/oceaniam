ALTER TABLE key_boxes
ADD CONSTRAINT ck_key_boxes_temporal_order
CHECK (
    activated_at < retired_at
    AND retired_at < expires_at
);
