# frontend

Flutter web app for the Oceaniam project. Managed with FVM (see `.fvmrc`,
Flutter 3.44.4). Run all Flutter/Dart commands through `fvm flutter ...` /
`fvm dart ...`.

## Why semantics are force-enabled

Flutter 3.44 ships only two web renderers, `canvaskit` (dart2js, default) and
`skwasm` (WebAssembly, `--wasm`). **Both render to a single `<canvas>` and emit
no semantic DOM by default.** This project must be drivable by the Firefox
DevTools MCP server (`@padenot/firefox-devtools-mcp`), which relies on DOM
snapshot/click/fill finders.

To make the canvas app accessible to browser automation, `lib/main.dart`
calls `SemanticsBinding.instance.ensureSemantics()` before `runApp`. This
forces Flutter to emit its ARIA-accessible semantics overlay (`<flt-semantics>`
elements with `role`, `tabindex`, and stable text labels) on top of the
canvas. The custom `web/flutter_bootstrap.js` also pins
`config.renderer: 'canvaskit'` so the renderer choice is stable.

Interactive widgets carry `Key`s (`appbar-title`, `counter-value`,
`increment-fab`) for stable identification.

## Running the app for Firefox MCP automation

```bash
# from frontend/
nohup fvm flutter run -d web-server --web-port 8099 --release \
  > /tmp/flutter_web.log 2>&1 &
```

The app is served at `http://localhost:8099`. `web-server` keeps it
headless and addressable by the Firefox MCP server without a Chrome
debugging extension.

## Firefox MCP workflow

1. `navigate_page` → `http://localhost:8099`
2. Wait ~6s for the canvaskit app to boot, then `take_snapshot`.
   You should see `<flt-semantics>` nodes:
   - `h2 "Flutter Demo Home Page"` (appbar title)
   - `span` with the counter value (e.g. `"0"`)
   - `button tag=flt-semantics text="Increment"` (FAB, role=button, flt-tappable)
3. `click_by_uid` on the `Increment` button; the counter increments.
4. `take_snapshot` again to confirm the counter text updated.

Finders that work: `ByText`, `BySemanticsLabel`, `ByValueKey` (matches the
`Key` values set in `lib/main.dart`).

## Notes

- Stale snapshots: after a click, the previous snapshot's UIDs/text may be
  cached. Re-run `take_snapshot` (or use `evaluate_script` to read the live
  `<flt-semantics>` text) before asserting the new counter value.
- The default `flutter run` build uses canvaskit; the custom bootstrap only
  makes the renderer choice explicit. To switch to skwasm, build with
  `--wasm` and change `config.renderer` to `'skwasm'` in
  `web/flutter_bootstrap.js`.

