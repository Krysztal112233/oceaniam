import 'dart:js_interop';

import 'package:flutter/foundation.dart';

@JS('window.location.origin')
external JSString? get _windowLocationOrigin;

/// Resolves the backend base URL.
///
/// 1. If `OCEANIAM_BACKEND_URL` was provided at compile time via `--dart-define`,
///    it is used as-is.
/// 2. Otherwise, on web, fall back to `${window.location.origin}/api` so the
///    same built artifact works behind any reverse-proxy / gateway.
/// 3. Outside web (or when JS interop is unavailable), use `http://localhost:8000`.
String resolveBackendBaseUrl() {
  const fromEnv = String.fromEnvironment('OCEANIAM_BACKEND_URL');
  if (fromEnv.isNotEmpty) return fromEnv;

  if (kIsWeb) {
    final origin = _windowLocationOrigin?.toDart;
    if (origin != null && origin.isNotEmpty) {
      return '$origin/api';
    }
  }

  return 'http://localhost:8000';
}
