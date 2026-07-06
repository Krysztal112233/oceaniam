import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

/// MFA challenge 二段验证页（TOTP / Email TOTP）。
///
/// 后端：POST /tenants/{tid}/applications/{aid}/challenges/{challenge_id}
class MfaChallengePage extends StatefulWidget {
  final String? challengeId;

  const MfaChallengePage({super.key, this.challengeId});

  @override
  State<MfaChallengePage> createState() => _MfaChallengePageState();
}

class _MfaChallengePageState extends State<MfaChallengePage> {
  final _codeController = TextEditingController();

  @override
  void dispose() {
    _codeController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 360),
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Icon(FluentIcons.shield_checkmark_24_regular, size: 40),
                const SizedBox(height: 12),
                Text(
                  'Two-factor verification',
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
                const SizedBox(height: 24),
                TextField(
                  key: const Key('mfa-code'),
                  controller: _codeController,
                  keyboardType: TextInputType.number,
                  decoration: const InputDecoration(
                    labelText: 'Verification code',
                    prefixIcon: Icon(FluentIcons.dialpad_24_regular),
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 16),
                FilledButton(
                  key: const Key('mfa-submit'),
                  onPressed: () {
                    // TODO: POST /challenges/{challenge_id} with payload.
                  },
                  child: const Text('Verify'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
