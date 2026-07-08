import 'package:flutter/material.dart';

import '../../widgets/placeholder_page.dart';

class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context) {
    return PlaceholderPage(
      title: 'Dashboard',
      description: 'Platform-wide overview with statistics and trend charts.',
    );
  }
}
