CREATE INDEX IF NOT EXISTS "idx_audits_payload_application_id" ON "audits" ((payload -> 'data' ->> 'application_id'));

CREATE INDEX IF NOT EXISTS "idx_audits_payload_tenant_id" ON "audits" ((payload -> 'data' ->> 'tenant_id'));

CREATE INDEX IF NOT EXISTS "idx_audits_payload_subject_id" ON "audits" ((payload -> 'data' ->> 'subject_id'));
