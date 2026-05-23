#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::pagination::PageInfo")]
pub struct PageInfo;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::pagination::PageParam")]
pub struct PageParam;
