import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import 'audience_list_field.dart';
import 'configuration_section.dart';

typedef SaveApplicationConfiguration =
    Future<bool> Function(PatchApplicationConfiguration patch);

/// Form for the complete application configuration schema.
///
/// Writable fields produce a minimal PATCH document. Argon2 parameters are
/// rendered as read-only because the backend exposes them in GET responses but
/// does not expose them in the PATCH request schema.
class ApplicationConfigurationEditor extends StatefulWidget {
  final ApplicationConfiguration configuration;
  final SaveApplicationConfiguration onSave;

  const ApplicationConfigurationEditor({
    super.key,
    required this.configuration,
    required this.onSave,
  });

  @override
  State<ApplicationConfigurationEditor> createState() =>
      _ApplicationConfigurationEditorState();
}

class _ApplicationConfigurationEditorState
    extends State<ApplicationConfigurationEditor> {
  final _formKey = GlobalKey<FormState>();
  late final TextEditingController _issuerController;
  late List<String> _audiences;
  late bool _registrationEnabled;

  late String _baselineIssuer;
  late List<String> _baselineAudiences;
  late bool _baselineRegistrationEnabled;

  bool _saving = false;
  bool _showAudienceError = false;

  @override
  void initState() {
    super.initState();
    final configuration = widget.configuration;
    _baselineIssuer = configuration.auth.token.issuer;
    _baselineAudiences = [...configuration.auth.token.audience];
    _baselineRegistrationEnabled = configuration.registration.enabled;
    _issuerController = TextEditingController(text: _baselineIssuer)
      ..addListener(_handleFieldChanged);
    _audiences = [..._baselineAudiences];
    _registrationEnabled = _baselineRegistrationEnabled;
  }

  @override
  void dispose() {
    _issuerController
      ..removeListener(_handleFieldChanged)
      ..dispose();
    super.dispose();
  }

  bool get _issuerChanged => _issuerController.text.trim() != _baselineIssuer;

  bool get _audiencesChanged => !listEquals(_audiences, _baselineAudiences);

  bool get _registrationChanged =>
      _registrationEnabled != _baselineRegistrationEnabled;

  bool get _isDirty =>
      _issuerChanged || _audiencesChanged || _registrationChanged;

  void _handleFieldChanged() => setState(() {});

  void _setAudiences(List<String> value) {
    setState(() {
      _audiences = value;
      if (value.isNotEmpty) _showAudienceError = false;
    });
  }

  void _reset() {
    _issuerController.text = _baselineIssuer;
    setState(() {
      _audiences = [..._baselineAudiences];
      _registrationEnabled = _baselineRegistrationEnabled;
      _showAudienceError = false;
    });
    _formKey.currentState?.reset();
  }

  Future<void> _save() async {
    final validForm = _formKey.currentState?.validate() ?? false;
    final validAudiences = _audiences.isNotEmpty;
    if (!validForm || !validAudiences) {
      setState(() => _showAudienceError = !validAudiences);
      return;
    }

    final tokenChanged = _issuerChanged || _audiencesChanged;
    final patch = PatchApplicationConfiguration(
      auth: tokenChanged
          ? PatchAuthConfiguration(
              token: PatchTokenConfiguration(
                issuer: _issuerChanged ? _issuerController.text.trim() : null,
                audience: _audiencesChanged ? [..._audiences] : null,
              ),
            )
          : null,
      registration: _registrationChanged
          ? PatchRegistrationConfiguration(enabled: _registrationEnabled)
          : null,
    );

    setState(() => _saving = true);
    final saved = await widget.onSave(patch);
    if (!mounted) return;
    setState(() {
      _saving = false;
      if (saved) {
        _baselineIssuer = _issuerController.text.trim();
        _baselineAudiences = [..._audiences];
        _baselineRegistrationEnabled = _registrationEnabled;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final configuration = widget.configuration;
    final theme = Theme.of(context);

    return Form(
      key: _formKey,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          LayoutBuilder(
            builder: (context, constraints) {
              final heading = Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Application configuration',
                    style: theme.textTheme.titleLarge,
                  ),
                  const SizedBox(height: 4),
                  Text(
                    'Control token validation, password hashing, and user registration.',
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
                ],
              );
              final actions = Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  OutlinedButton(
                    key: const Key('configuration-reset'),
                    onPressed: _isDirty && !_saving ? _reset : null,
                    child: const Text('Reset'),
                  ),
                  const SizedBox(width: 8),
                  FilledButton.icon(
                    key: const Key('configuration-save'),
                    onPressed: _isDirty && !_saving ? _save : null,
                    icon: _saving
                        ? const SizedBox.square(
                            dimension: 16,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(FluentIcons.save_24_regular, size: 18),
                    label: Text(_saving ? 'Saving' : 'Save changes'),
                  ),
                ],
              );

              if (constraints.maxWidth < 560) {
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    heading,
                    const SizedBox(height: 12),
                    Align(alignment: Alignment.centerRight, child: actions),
                  ],
                );
              }

              return Row(
                children: [
                  Expanded(child: heading),
                  const SizedBox(width: 12),
                  actions,
                ],
              );
            },
          ),
          const SizedBox(height: 16),
          ConfigurationSection(
            icon: FluentIcons.key_24_regular,
            title: 'Token',
            description:
                'Issuer and audiences required when validating application JWTs.',
            children: [
              TextFormField(
                key: const Key('configuration-token-issuer'),
                controller: _issuerController,
                enabled: !_saving,
                decoration: const InputDecoration(
                  labelText: 'Issuer',
                  helperText: 'Expected value of the JWT iss claim.',
                  border: OutlineInputBorder(),
                ),
                validator: (value) => value == null || value.trim().isEmpty
                    ? 'Issuer is required.'
                    : null,
              ),
              const SizedBox(height: 16),
              AudienceListField(
                value: _audiences,
                enabled: !_saving,
                errorText: _showAudienceError
                    ? 'At least one audience is required.'
                    : null,
                onChanged: _setAudiences,
              ),
            ],
          ),
          const SizedBox(height: 16),
          ConfigurationSection(
            icon: FluentIcons.lock_closed_24_regular,
            title: 'Password hashing',
            description:
                'Argon2id parameters used when hashing new application-user passwords.',
            trailing: const Chip(label: Text('Read only')),
            children: [
              LayoutBuilder(
                builder: (context, constraints) {
                  final fields = [
                    _ReadOnlyNumberField(
                      key: const Key('configuration-argon2-memory'),
                      label: 'Memory cost',
                      value: configuration.auth.password.argon2.mCost,
                      suffix: 'KiB',
                    ),
                    _ReadOnlyNumberField(
                      key: const Key('configuration-argon2-time'),
                      label: 'Time cost',
                      value: configuration.auth.password.argon2.tCost,
                      suffix: 'iterations',
                    ),
                    _ReadOnlyNumberField(
                      key: const Key('configuration-argon2-parallelism'),
                      label: 'Parallelism',
                      value: configuration.auth.password.argon2.pCost,
                      suffix: 'lanes',
                    ),
                  ];
                  if (constraints.maxWidth < 620) {
                    return Column(
                      children: [
                        for (var index = 0; index < fields.length; index++) ...[
                          fields[index],
                          if (index != fields.length - 1)
                            const SizedBox(height: 12),
                        ],
                      ],
                    );
                  }

                  return Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      for (var index = 0; index < fields.length; index++) ...[
                        Expanded(child: fields[index]),
                        if (index != fields.length - 1)
                          const SizedBox(width: 12),
                      ],
                    ],
                  );
                },
              ),
            ],
          ),
          const SizedBox(height: 16),
          ConfigurationSection(
            icon: FluentIcons.person_add_24_regular,
            title: 'Registration',
            description: 'Choose whether this application accepts new users.',
            children: [
              SwitchListTile.adaptive(
                key: const Key('configuration-registration-enabled'),
                contentPadding: EdgeInsets.zero,
                title: const Text('Allow user registration'),
                subtitle: const Text(
                  'When disabled, existing users can still sign in.',
                ),
                value: _registrationEnabled,
                onChanged: _saving
                    ? null
                    : (value) => setState(() => _registrationEnabled = value),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _ReadOnlyNumberField extends StatelessWidget {
  final String label;
  final int value;
  final String suffix;

  const _ReadOnlyNumberField({
    super.key,
    required this.label,
    required this.value,
    required this.suffix,
  });

  @override
  Widget build(BuildContext context) {
    return TextFormField(
      initialValue: value.toString(),
      readOnly: true,
      canRequestFocus: false,
      decoration: InputDecoration(
        labelText: label,
        suffixText: suffix,
        filled: true,
        border: const OutlineInputBorder(),
      ),
    );
  }
}
