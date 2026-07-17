import 'package:flutter/material.dart';

class AdminPageScaffold extends StatelessWidget {
  final String title;
  final List<Widget>? leading;
  final List<Widget>? actions;
  final double widthFactor;
  final Widget child;

  const AdminPageScaffold({
    super.key,
    required this.title,
    required this.child,
    this.leading,
    this.actions,
    this.widthFactor = 0.7,
  });

  @override
  Widget build(BuildContext context) {
    final isWide = MediaQuery.of(context).size.width >= 900;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(24, 16, 24, 8),
          child: Row(
            children: [
              ...?leading,
              if (leading != null) const SizedBox(width: 12),
              Text(title, style: Theme.of(context).textTheme.headlineSmall),
              const Spacer(),
              ...?actions,
            ],
          ),
        ),
        Expanded(
          child: isWide
              ? Center(
                  child: FractionallySizedBox(
                    widthFactor: widthFactor,
                    child: child,
                  ),
                )
              : child,
        ),
      ],
    );
  }
}
