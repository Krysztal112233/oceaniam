INSERT INTO
  credentials (id, phc)
SELECT
  id,
  phc
FROM
  administrators a
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      credentials c
    WHERE
      c.id = a.id
  );

ALTER TABLE administrators
DROP COLUMN phc;
