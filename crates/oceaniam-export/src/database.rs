#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_database::model::subjects::Model")]
pub struct Subjects;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_database::model::users::Model")]
pub struct Users;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_database::model::credentials::Model")]
pub struct Credentials;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_database::model::key_boxes::Model")]
pub struct KeyBoxes;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_database::model::revoked_jwts::Model")]
pub struct RevokedJwts;
