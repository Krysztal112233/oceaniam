{{flutter_js}}
{{flutter_build_config}}

_flutter.loader.load({
  serviceWorkerSettings: {
    serviceWorkerVersion: {{flutter_service_worker_version}},
  },
  config: {
    // Pin the renderer to canvaskit (dart2js). Flutter 3.44 only ships
    // canvaskit and skwasm; both render to <canvas>. We keep canvaskit
    // for dart2js builds and rely on Flutter's semantics accessibility
    // overlay (enabled in main.dart via SemanticsBinding.ensureSemantics)
    // to expose an ARIA DOM tree that browser automation can drive.
    renderer: 'canvaskit',
    // 加载本地随构建拷贝的 canvaskit，而非 gstatic.com CDN，
    // 便于离线/受限网络环境（headless Firefox 自动化）。
    canvasKitBaseUrl: '/canvaskit/',
  },
});