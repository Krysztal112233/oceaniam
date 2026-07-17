import 'package:flutter/material.dart';

/// Table cell text: single-line ellipsis with a tooltip showing the full
/// value. Set [mono] for identifier columns.
class TableCellText extends StatelessWidget {
  final String text;
  final bool mono;

  const TableCellText(this.text, {super.key, this.mono = false});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Align(
        alignment: Alignment.centerLeft,
        child: Tooltip(
          message: text,
          child: Text(
            text,
            overflow: TextOverflow.ellipsis,
            style: mono
                ? Theme.of(
                    context,
                  ).textTheme.bodyMedium?.copyWith(fontFamily: 'monospace')
                : null,
          ),
        ),
      ),
    );
  }
}
