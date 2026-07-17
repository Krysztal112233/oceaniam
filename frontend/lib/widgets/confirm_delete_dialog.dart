import 'package:flutter/material.dart';
import 'package:word_generator/word_generator.dart';

/// A reusable delete-confirmation dialog that asks the user to type two
/// randomly generated words before the delete action is enabled.
///
/// Returns `true` if the user confirmed and typed the phrase correctly,
/// otherwise `false`.
class ConfirmDeleteDialog extends StatefulWidget {
  final String title;
  final String itemName;
  final String confirmButtonText;

  const ConfirmDeleteDialog({
    super.key,
    required this.title,
    required this.itemName,
    required this.confirmButtonText,
  });

  @override
  State<ConfirmDeleteDialog> createState() => _ConfirmDeleteDialogState();
}

class _ConfirmDeleteDialogState extends State<ConfirmDeleteDialog> {
  final _controller = TextEditingController();
  late final String _phrase;

  @override
  void initState() {
    super.initState();
    final generator = WordGenerator();
    final first = _capitalize(generator.randomNoun());
    final second = _capitalize(generator.randomNoun());
    _phrase = '$first $second';
    _controller.addListener(() => setState(() {}));
  }

  static String _capitalize(String word) {
    if (word.isEmpty) return word;
    return word[0].toUpperCase() + word.substring(1);
  }

  bool get _matches => _controller.text.trim() == _phrase;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return AlertDialog(
      title: Text(widget.title),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            'Are you sure you want to delete "${widget.itemName}"? This action cannot be undone.',
          ),
          const SizedBox(height: 16),
          Text(
            'To confirm, type the following words:',
            style: theme.textTheme.bodyMedium?.copyWith(
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 8),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: theme.colorScheme.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text(
              _phrase,
              style: theme.textTheme.bodyLarge?.copyWith(
                fontWeight: FontWeight.bold,
                letterSpacing: 0.5,
              ),
              textAlign: TextAlign.center,
            ),
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _controller,
            autofocus: true,
            decoration: const InputDecoration(hintText: 'Type the words above'),
            onSubmitted: (_) => _matches ? _confirm() : null,
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(false),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: _matches ? _confirm : null,
          style: FilledButton.styleFrom(
            backgroundColor: theme.colorScheme.error,
          ),
          child: Text(widget.confirmButtonText),
        ),
      ],
    );
  }

  void _confirm() {
    Navigator.of(context).pop(true);
  }
}
