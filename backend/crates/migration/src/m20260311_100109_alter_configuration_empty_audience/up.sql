UPDATE applications
SET configuration = jsonb_set(
    configuration,
    '{authentication,audience}',
    '["OceanIAM"]'::jsonb,
    true
)
WHERE configuration #> '{authentication,audience}' = '[]'::jsonb
