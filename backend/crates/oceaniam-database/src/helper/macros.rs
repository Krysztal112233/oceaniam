/// Generates the body of `is_exist(id, db)` for a system-protected entity.
#[macro_export]
macro_rules! system_protected_is_exist {
    ($Entity:ty, $SYSTEM_ID:expr, $id:expr, $database:expr) => {{
        if $id == $SYSTEM_ID {
            return Ok(false);
        }
        Ok(<$Entity>::find_by_id($id).one($database).await?.is_some())
    }};
}

/// Generates the body of `is_system_*_exist(db)` for a system-protected entity.
#[macro_export]
macro_rules! system_protected_is_system_exist {
    ($Entity:ty, $SYSTEM_ID:expr, $database:expr) => {{
        Ok(<$Entity>::find_by_id($SYSTEM_ID)
            .one($database)
            .await?
            .is_some())
    }};
}

/// Generates the body of `get_*(id, db)` for a system-protected entity.
/// `$not_found` is an expression `\|id\| Error` that creates the not-found error.
#[macro_export]
macro_rules! system_protected_get {
    ($Entity:ty, $SYSTEM_ID:expr, $id:expr, $database:expr, $not_found:expr) => {{
        if $id == $SYSTEM_ID {
            return Err($not_found($id));
        }
        <$Entity>::find_by_id($id)
            .one($database)
            .await?
            .ok_or_else(|| $not_found($id))
    }};
}

/// Generates the body of `get_system_*(db)` for a system-protected entity.
#[macro_export]
macro_rules! system_protected_get_system {
    ($Entity:ty, $SYSTEM_ID:expr, $database:expr, $not_found:expr) => {{
        <$Entity>::find_by_id($SYSTEM_ID)
            .one($database)
            .await?
            .ok_or_else(|| $not_found($SYSTEM_ID))
    }};
}

/// Generates the body of `delete_*(id, db)` for a system-protected entity.
#[macro_export]
macro_rules! system_protected_delete {
    ($Entity:ty, $SYSTEM_ID:expr, $id:expr, $database:expr, $not_found:expr) => {{
        if $id == $SYSTEM_ID {
            return Err($not_found($id));
        }
        <$Entity>::delete_by_id($id).exec($database).await?;
        Ok(())
    }};
}
