CREATE TYPE oidc_client_type AS ENUM ('public');
CREATE TYPE oidc_application_type AS ENUM ('web');

ALTER TABLE oidc_clients
    DROP CONSTRAINT oidc_clients_client_type_check,
    DROP CONSTRAINT oidc_clients_application_type_check,
    ALTER COLUMN client_type TYPE oidc_client_type
        USING client_type::text::oidc_client_type,
    ALTER COLUMN application_type TYPE oidc_application_type
        USING application_type::text::oidc_application_type;
