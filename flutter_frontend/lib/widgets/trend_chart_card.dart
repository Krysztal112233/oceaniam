import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

/// 单系列趋势折线图卡片（带面积填充）。
///
/// 接收一组 [TrendDataPoint]（bucket + count），按时间顺序绘制为
/// 平滑曲线 + 下方半透明色块。X 轴标签为 MM-DD，Y 轴自适应。
class TrendChartCard extends StatelessWidget {
  final String title;
  final List<TrendDataPoint> points;
  final Color color;
  final double height;

  const TrendChartCard({
    super.key,
    required this.title,
    required this.points,
    required this.color,
    this.height = 220,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final filled = points.where((p) => p.count > 0).toList();
    final hasData = points.isNotEmpty;

    return Card(
      elevation: 0,
      shape: RoundedRectangleBorder(
        side: BorderSide(color: theme.dividerColor),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Padding(
        padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  width: 10,
                  height: 10,
                  decoration: BoxDecoration(
                    color: color,
                    shape: BoxShape.circle,
                  ),
                ),
                const SizedBox(width: 8),
                Text(title, style: theme.textTheme.titleSmall),
              ],
            ),
            const SizedBox(height: 12),
            SizedBox(
              height: height,
              child: !hasData
                  ? _EmptyState(theme: theme)
                  : LineChart(
                      LineChartData(
                        minX: 0,
                        maxX: (points.length - 1).toDouble(),
                        minY: 0,
                        maxY:
                            (_maxCount(filled.isNotEmpty ? filled : points) *
                                    1.2)
                                .clamp(1.0, double.infinity),
                        lineBarsData: [
                          LineChartBarData(
                            spots: _toSpots(points),
                            isCurved: true,
                            curveSmoothness: 0.35,
                            color: color,
                            barWidth: 2.0,
                            isStrokeCapRound: true,
                            dotData: const FlDotData(show: false),
                            belowBarData: BarAreaData(
                              show: true,
                              color: color.withValues(alpha: 0.12),
                            ),
                          ),
                        ],
                        titlesData: FlTitlesData(
                          leftTitles: AxisTitles(
                            sideTitles: SideTitles(
                              showTitles: true,
                              reservedSize: 36,
                              interval: _yInterval(
                                filled.isNotEmpty ? filled : points,
                              ),
                              getTitlesWidget: (v, meta) => SideTitleWidget(
                                axisSide: meta.axisSide,
                                child: Text(
                                  v.toInt().toString(),
                                  style: theme.textTheme.bodySmall,
                                ),
                              ),
                            ),
                          ),
                          bottomTitles: AxisTitles(
                            sideTitles: SideTitles(
                              showTitles: true,
                              reservedSize: 28,
                              interval: _xInterval(points.length),
                              getTitlesWidget: (v, meta) => SideTitleWidget(
                                axisSide: meta.axisSide,
                                child: Text(
                                  _formatBucket(points, v.toInt()),
                                  style: theme.textTheme.bodySmall,
                                ),
                              ),
                            ),
                          ),
                          rightTitles: const AxisTitles(
                            sideTitles: SideTitles(showTitles: false),
                          ),
                          topTitles: const AxisTitles(
                            sideTitles: SideTitles(showTitles: false),
                          ),
                        ),
                        gridData: FlGridData(
                          show: true,
                          drawVerticalLine: false,
                          horizontalInterval: _yInterval(
                            filled.isNotEmpty ? filled : points,
                          ),
                          getDrawingHorizontalLine: (v) => FlLine(
                            color: theme.dividerColor.withValues(alpha: 0.4),
                            strokeWidth: 1,
                          ),
                        ),
                        borderData: FlBorderData(show: false),
                        lineTouchData: LineTouchData(
                          touchTooltipData: LineTouchTooltipData(
                            getTooltipItems: (spots) {
                              return spots.map((s) {
                                final p = points[s.spotIndex];
                                return LineTooltipItem(
                                  '${_formatDate(p.bucket)}\n${p.count}',
                                  TextStyle(
                                    color: color,
                                    fontWeight: FontWeight.w600,
                                    fontSize: 12,
                                  ),
                                );
                              }).toList();
                            },
                          ),
                        ),
                      ),
                    ),
            ),
          ],
        ),
      ),
    );
  }

  List<FlSpot> _toSpots(List<TrendDataPoint> pts) {
    return List.generate(
      pts.length,
      (i) => FlSpot(i.toDouble(), pts[i].count.toDouble()),
    );
  }

  int _maxCount(List<TrendDataPoint> pts) {
    return pts.fold<int>(0, (m, p) => p.count > m ? p.count : m);
  }

  double _yInterval(List<TrendDataPoint> pts) {
    final max = _maxCount(pts);
    if (max <= 4) return 1;
    if (max <= 20) return 5;
    if (max <= 100) return 20;
    if (max <= 1000) return 200;
    return (max / 5).ceilToDouble();
  }

  double _xInterval(int len) {
    if (len <= 7) return 1;
    if (len <= 30) return 5;
    return (len / 6).ceilToDouble();
  }

  String _formatBucket(List<TrendDataPoint> pts, int idx) {
    if (idx < 0 || idx >= pts.length) return '';
    return _formatDate(pts[idx].bucket);
  }

  String _formatDate(DateTime d) {
    final m = d.month.toString().padLeft(2, '0');
    final day = d.day.toString().padLeft(2, '0');
    return '$m-$day';
  }
}

class _EmptyState extends StatelessWidget {
  final ThemeData theme;
  const _EmptyState({required this.theme});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Text(
        'No data in range',
        style: theme.textTheme.bodyMedium?.copyWith(
          color: theme.colorScheme.onSurfaceVariant,
        ),
      ),
    );
  }
}
