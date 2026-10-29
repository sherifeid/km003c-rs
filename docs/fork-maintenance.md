# Maintaining this fork

The upstream-owned `km003c-lib`, `km003c-egui`, and `km003c-cli` directories
match the upstream commit merged into this branch. Do not add scope behavior
or manual-updater methods to those crates.

## Local additions

- `km003c-scope`: native and browser UI, recording files and console.
- `km003c-scope-core`: scope session worker, periodic ADC telemetry, browser
  polling policy, passive cable identity decoder and associated PD presentation.
  It depends on the public `km003c-lib` API. Its session owns the USB connection;
  never run the upstream session concurrently against the same meter.
- `km003c-firmware-manual`: standalone local-package updater library and CLI.
  `ManualUpdater` wraps an upstream device handle; package parsing and transfer
  behavior remain local. Upstream's catalog/GUI updater remains unchanged.

Upstream's bounded MemoryRead implementation supplies plaintext/encrypted
handling to all applications. There is no fork-specific memory reader.
The original live-meter GUI uses upstream behavior; custom telemetry remains
in the scope.

The scope's PD decoder/logger and session were extracted from this fork.
They are maintained locally and do not automatically acquire upstream session
or decoder changes. Review upstream fixes to those areas when syncing.

## Updating

Fetch upstream and merge `upstream/main` on an integration branch. Resolve
workspace members, lockfiles, documentation and CI by retaining local additions.
Avoid squash-merging sync PRs so upstream ancestry stays intact.

Check upstream-owned directory drift with:

```sh
git diff upstream/main -- km003c-lib km003c-egui km003c-cli
```

When comparing against a newer upstream, use the exact integrated commit
instead of `upstream/main` until the new merge is complete.

Validate local additions:

```sh
cargo test -p km003c-scope-core -p km003c-firmware-manual -p km003c-scope --locked
cargo check -p km003c-scope --target wasm32-unknown-unknown --locked
cargo run -p km003c-firmware-manual -- --help
```

These checks do not flash hardware. Firmware transfer and live native/browser
acquisition require separate hardware validation.
