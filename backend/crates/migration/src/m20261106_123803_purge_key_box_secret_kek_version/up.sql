UPDATE key_boxes
SET
  secret = secret - 'key_version'
WHERE
  secret ? 'key_version';
