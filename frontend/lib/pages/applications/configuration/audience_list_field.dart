import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';

/// List editor for JWT audience values.
///
/// The committed values are owned by the parent; this widget only owns the
/// draft input used to append a value.
class AudienceListField extends StatefulWidget {
  final List<String> value;
  final ValueChanged<List<String>> onChanged;
  final bool enabled;
  final String? errorText;

  const AudienceListField({
    super.key,
    required this.value,
    required this.onChanged,
    this.enabled = true,
    this.errorText,
  });

  @override
  State<AudienceListField> createState() => _AudienceListFieldState();
}

class _AudienceListFieldState extends State<AudienceListField> {
  final _controller = TextEditingController();
  String? _draftError;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _addAudience() {
    final value = _controller.text.trim();
    if (value.isEmpty) {
      setState(() => _draftError = 'Enter an audience value.');
      return;
    }
    if (widget.value.contains(value)) {
      setState(() => _draftError = 'This audience is already present.');
      return;
    }

    widget.onChanged([...widget.value, value]);
    _controller.clear();
    setState(() => _draftError = null);
  }

  void _removeAudience(int index) {
    final updated = [...widget.value]..removeAt(index);
    widget.onChanged(updated);
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Semantics(
      container: true,
      label: 'Token audiences',
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('Audience', style: theme.textTheme.labelLarge),
          const SizedBox(height: 8),
          if (widget.value.isEmpty)
            Text(
              'No audience values configured.',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.error,
              ),
            )
          else
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                for (var index = 0; index < widget.value.length; index++)
                  InputChip(
                    key: ValueKey('audience-${widget.value[index]}'),
                    label: Text(widget.value[index]),
                    onDeleted: widget.enabled
                        ? () => _removeAudience(index)
                        : null,
                    deleteIcon: const Icon(
                      FluentIcons.dismiss_16_regular,
                      size: 16,
                    ),
                  ),
              ],
            ),
          const SizedBox(height: 12),
          TextField(
            key: const Key('configuration-audience-input'),
            controller: _controller,
            enabled: widget.enabled,
            onSubmitted: (_) => _addAudience(),
            decoration: InputDecoration(
              labelText: 'Add audience',
              hintText: 'For example, api.example.com',
              helperText: 'Values accepted in the JWT aud claim.',
              errorText: _draftError ?? widget.errorText,
              border: const OutlineInputBorder(),
              suffixIcon: IconButton(
                key: const Key('configuration-audience-add'),
                tooltip: 'Add audience',
                onPressed: widget.enabled ? _addAudience : null,
                icon: const Icon(FluentIcons.add_24_regular),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
