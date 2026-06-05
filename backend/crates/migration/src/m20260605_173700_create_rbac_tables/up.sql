INSERT INTO role_permissions (role_id, permission)
SELECT ar.id, unnest(ARRAY['application_user_read', 'application_challenge_read'])
FROM application_roles ar
WHERE ar.name = 'reader' AND ar.is_system = true
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_id, permission)
SELECT ar.id, unnest(ARRAY['application_user_read', 'application_challenge_read', 'application_token_issue'])
FROM application_roles ar
WHERE ar.name = 'member' AND ar.is_system = true
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_id, permission)
SELECT ar.id, unnest(ARRAY['application_user_read', 'application_challenge_read', 'application_token_issue', 'application_user_invite', 'application_user_patch', 'application_token_revoke'])
FROM application_roles ar
WHERE ar.name = 'admin' AND ar.is_system = true
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_id, permission)
SELECT ar.id, unnest(ARRAY['application_user_read', 'application_challenge_read', 'application_token_issue', 'application_user_invite', 'application_user_patch', 'application_token_revoke', 'application_user_delete'])
FROM application_roles ar
WHERE ar.name = 'owner' AND ar.is_system = true
ON CONFLICT DO NOTHING;
