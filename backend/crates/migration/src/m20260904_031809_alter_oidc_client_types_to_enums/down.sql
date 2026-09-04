ALTER TABLE oidc_clients
    ALTER COLUMN client_type TYPE VARCHAR(16)
        USING client_type::text,
    ALTER COLUMN application_type TYPE VARCHAR(16)
        USING application_type::text,
    ADD CONSTRAINT oidc_clients_client_type_check
        CHECK (client_type = 'public'),
    ADD CONSTRAINT oidc_clients_application_type_check
        CHECK (application_type = 'web');

DROP TYPE oidc_application_type;
DROP TYPE oidc_client_type;
