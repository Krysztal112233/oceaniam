import 'package:flutter/material.dart';
import 'package:oceaniam_sdk/oceaniam_sdk.dart';

import 'application_user_detail_page.dart';

const _detailSheetBreakpoint = 900.0;
const _detailSheetWidth = 520.0;

Future<bool?> showApplicationUserDetail(
  BuildContext context, {
  required String tenantId,
  required String applicationId,
  required ApplicationUser user,
}) {
  final detailPage = ApplicationUserDetailPage(
    tenantId: tenantId,
    applicationId: applicationId,
    user: user,
  );

  if (MediaQuery.sizeOf(context).width < _detailSheetBreakpoint) {
    return Navigator.of(
      context,
    ).push<bool>(MaterialPageRoute(builder: (context) => detailPage));
  }

  return showGeneralDialog<bool>(
    context: context,
    barrierDismissible: true,
    barrierLabel: 'Close user details',
    barrierColor: Colors.black54,
    transitionDuration: const Duration(milliseconds: 250),
    pageBuilder: (context, animation, secondaryAnimation) => Align(
      alignment: Alignment.centerRight,
      child: SizedBox(
        key: const Key('application-user-detail-sheet'),
        width: _detailSheetWidth,
        height: double.infinity,
        child: Material(
          color: Theme.of(context).colorScheme.surface,
          child: SafeArea(child: detailPage),
        ),
      ),
    ),
    transitionBuilder: (context, animation, secondaryAnimation, child) {
      final offset = Tween<Offset>(
        begin: const Offset(1, 0),
        end: Offset.zero,
      ).chain(CurveTween(curve: Curves.easeOutCubic)).animate(animation);
      return SlideTransition(position: offset, child: child);
    },
  );
}
