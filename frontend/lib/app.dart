import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'pages/auth/login_page.dart';
import 'providers/auth_controller.dart';
import 'shells/admin_shell.dart';
import 'theme/theme_controller.dart';

class OceanIAMApp extends ConsumerStatefulWidget {
  const OceanIAMApp({super.key});

  @override
  ConsumerState<OceanIAMApp> createState() => _OceanIAMAppState();
}

class _OceanIAMAppState extends ConsumerState<OceanIAMApp> {
  bool _sessionRestored = false;

  @override
  void initState() {
    super.initState();
    Future.microtask(_restore);
  }

  Future<void> _restore() async {
    await ref.read(authControllerProvider.notifier).restoreSession();
    if (mounted) setState(() => _sessionRestored = true);
  }

  @override
  Widget build(BuildContext context) {
    final themeMode = ref.watch(themeControllerProvider);
    final authState = ref.watch(authControllerProvider);

    return MaterialApp(
      title: 'OceanIAM Admin',
      theme: ThemeController.light(),
      darkTheme: ThemeController.dark(),
      themeMode: themeMode,
      home: !_sessionRestored || authState is AuthLoading
          ? const _Splash()
          : authState is AuthAuthenticated
          ? const AdminShell()
          : const LoginPage(),
    );
  }
}

class _Splash extends StatelessWidget {
  const _Splash();

  @override
  Widget build(BuildContext context) {
    return const Scaffold(body: Center(child: CircularProgressIndicator()));
  }
}
