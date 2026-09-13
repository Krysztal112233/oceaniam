use sea_orm::DbErr;

use super::Error;

// NOTE: AI-generated test
#[test]
fn from_conversion_records_the_conversion_callsite() {
    let expected_line = line!() + 1;
    let error = Error::from(DbErr::Custom("test error".to_owned()));

    let Error::Db { location, .. } = error else {
        panic!("expected database error");
    };

    assert!(location.file.ends_with("error_location_tests.rs"));
    assert_eq!(location.line, expected_line);
}
