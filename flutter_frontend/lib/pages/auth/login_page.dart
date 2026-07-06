import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

/// 平台管理员登录页（name + password）。
///
/// 后端：POST /auth/tokens
/// 若返回 MFA challenge，跳转 [MfaChallengePage]。
class LoginPage extends StatefulWidget {
  const LoginPage({super.key});

  @override
  State<LoginPage> createState() => _LoginPageState();
}

class _LoginPageState extends State<LoginPage> {
  final _nameController = TextEditingController();
  final _passwordController = TextEditingController();

  @override
  void dispose() {
    _nameController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Scaffold(
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 360),
          child: Card(
            elevation: 0,
            shape: RoundedRectangleBorder(
              side: BorderSide(color: theme.dividerColor),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Icon(
                    FluentIcons.shield_keyhole_24_regular,
                    size: 40,
                    color: theme.colorScheme.primary,
                  ),
                  const SizedBox(height: 12),
                  Text(
                    'OceanIAM Admin',
                    style: theme.textTheme.headlineSmall,
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 24),
                  TextField(
                    key: const Key('login-name'),
                    controller: _nameController,
                    decoration: const InputDecoration(
                      labelText: 'Administrator name',
                      prefixIcon: Icon(FluentIcons.person_24_regular),
                      border: OutlineInputBorder(),
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    key: const Key('login-password'),
                    controller: _passwordController,
                    obscureText: true,
                    decoration: const InputDecoration(
                      labelText: 'Password',
                      prefixIcon: Icon(FluentIcons.lock_closed_24_regular),
                      border: OutlineInputBorder(),
                    ),
                  ),
                  const SizedBox(height: 16),
                  FilledButton(
                    key: const Key('login-submit'),
                    onPressed: () {
                      // TODO: POST /auth/tokens, dispatch cookie|json|both.
                    },
                    child: const Text('Sign in'),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
