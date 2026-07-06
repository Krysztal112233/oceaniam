import 'package:flutter/material.dart';

import 'pages/auth/login_page.dart';
import 'shells/admin_shell.dart';

class OceanIAMApp extends StatelessWidget {
  const OceanIAMApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'OceanIAM Admin',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.indigo),
        useMaterial3: true,
      ),
      initialRoute: '/',
      routes: {
        '/': (_) => const AdminShell(),
        '/login': (_) => const LoginPage(),
      },
    );
  }
}
