UPDATE administrators SET role = 'super_admin' WHERE name = 'root';

INSERT INTO application_roles (id, application_id, name, is_system)
SELECT gen_random_uuid(), id, unnest(ARRAY['owner', 'admin', 'member', 'reader']), true
FROM applications;
