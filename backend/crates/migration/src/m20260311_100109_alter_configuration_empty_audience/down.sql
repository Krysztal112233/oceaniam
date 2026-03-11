UPDATE applications
SET configuration = jsonb_set(
    configuration,
    '{authentication,audience}',
    '[]'::jsonb,
    true
)
WHERE configuration #> '{authentication,audience}' = '["OceanIAM"]'::jsonb
