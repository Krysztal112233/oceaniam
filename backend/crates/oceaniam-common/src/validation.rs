pub fn forbid_search_wildcards(value: &Option<String>, _: &()) -> garde::Result {
    if let Some(value) = value.as_deref()
        && (value.contains('%') || value.contains('_') || value.contains('\\'))
    {
        return Err(garde::Error::new(
            "must not contain expressions that expand `LIKE/ILIKE` search scope",
        ));
    }

    Ok(())
}
