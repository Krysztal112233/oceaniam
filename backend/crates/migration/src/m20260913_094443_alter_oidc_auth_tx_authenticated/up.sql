-- `subject_id` references the identity root `subjects(id)` (the same id as `users.id`),
-- never `users.oidc_sub`. ON DELETE CASCADE mirrors the table's owner-cascade discipline:
-- removing a subject tears down its in-flight authentication snapshots.
ALTER TABLE oidc_authorization_transactions
ADD CONSTRAINT fk_oidc_auth_tx_subject
FOREIGN KEY (subject_id) REFERENCES subjects (id)
ON UPDATE NO ACTION ON DELETE CASCADE;
