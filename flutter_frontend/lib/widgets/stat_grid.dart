import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

import '../models/stat.dart';
import '../widgets/stat_card.dart';

class StatGrid extends StatelessWidget {
  const StatGrid({super.key});

  static const _stats = [
    Stat('Users', '1,284', FluentIcons.people_24_regular),
    Stat('Revenue', r'$48.2k', FluentIcons.money_24_regular),
    Stat('Orders', '342', FluentIcons.cart_24_regular),
    Stat('Uptime', '99.9%', FluentIcons.flash_24_regular),
  ];

  @override
  Widget build(BuildContext context) {
    return GridView.count(
      crossAxisCount: MediaQuery.of(context).size.width >= 600 ? 4 : 2,
      padding: const EdgeInsets.all(16),
      crossAxisSpacing: 16,
      mainAxisSpacing: 16,
      childAspectRatio: 1.4,
      children: _stats.map((s) => StatCard(s)).toList(),
    );
  }
}
