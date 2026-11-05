ALTER TABLE key_boxes
DROP CONSTRAINT IF EXISTS ck_key_boxes_kek_version_positive;

ALTER TABLE key_boxes
DROP COLUMN IF EXISTS kek_version;
