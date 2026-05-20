#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::statistics::OverviewVO")]
pub struct OverviewVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::statistics::AuditLogVO")]
pub struct AuditLogVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::statistics::ApplicationStatisticsVO")]
pub struct ApplicationStatisticsVO;
