import 'package:flutter/material.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import '../../widgets/expandable_list_card.dart';
import 'secret_expanded_panel.dart';

class SecretCard extends StatelessWidget {
  final Secret secret;
  final bool isExpanded;
  final VoidCallback? onExpand;

  const SecretCard({
    super.key,
    required this.secret,
    required this.isExpanded,
    this.onExpand,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final revoked = secret.revokedAt != null;
    final bindingCount = secret.applicationIds.length;

    return ExpandableListCard(
      leading: CircleAvatar(
        backgroundColor: revoked
            ? theme.colorScheme.errorContainer
            : theme.colorScheme.secondaryContainer,
        child: Icon(
          FluentIcons.key_24_regular,
          color: revoked
              ? theme.colorScheme.onErrorContainer
              : theme.colorScheme.onSecondaryContainer,
        ),
      ),
      title: Text(secret.id, maxLines: 1, overflow: TextOverflow.ellipsis),
      subtitle: Text(
        [
          secret.secret ?? '—',
          secret.createdAt,
          if (revoked) 'revoked',
          '$bindingCount binding${bindingCount == 1 ? '' : 's'}',
        ].join(' · '),
        maxLines: 2,
        overflow: TextOverflow.ellipsis,
      ),
      isExpanded: isExpanded,
      onExpand: onExpand,
      expandedChild: SecretExpandedPanel(secret: secret),
    );
  }
}
