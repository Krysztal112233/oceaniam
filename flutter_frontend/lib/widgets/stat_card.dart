import 'package:flutter/material.dart';

import '../models/stat.dart';

class StatCard extends StatelessWidget {
  final Stat stat;

  const StatCard(this.stat, {super.key});

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 0,
      shape: RoundedRectangleBorder(
        side: BorderSide(color: Theme.of(context).dividerColor),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Row(
              children: [
                Icon(
                  stat.icon,
                  size: 20,
                  color: Theme.of(context).colorScheme.primary,
                ),
                const SizedBox(width: 8),
                Text(stat.label, style: Theme.of(context).textTheme.labelLarge),
              ],
            ),
            Text(stat.value, style: Theme.of(context).textTheme.headlineMedium),
          ],
        ),
      ),
    );
  }
}
