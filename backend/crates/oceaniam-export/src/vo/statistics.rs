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

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::statistics::TrendDataPoint")]
pub struct TrendDataPoint;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::statistics::Granularity")]
pub struct Granularity;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::statistics::PlatformTrendsVO")]
pub struct PlatformTrendsVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::statistics::ApplicationTrendsVO")]
pub struct ApplicationTrendsVO;
