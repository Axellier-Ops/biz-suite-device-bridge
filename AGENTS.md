# Biz Suite Windows Agent Context

Read `../AGENTS.md` as the workspace-level context when available.

This repo is the Windows native shell for Biz Suite Cloud POS. It is a Tauri + Vite app that loads the cloud POS and adds native Windows printer and cash drawer support.

## Important Areas

- `src-tauri/` contains Rust/Tauri native capabilities.
- `src-tauri/src/main.rs` contains bridge commands, printer routing, ESC/POS receipt/KOT rendering, pairing, polling, and update checks.
- `src/` contains the local shell UI.
- `scripts/` contains build/release helpers.

## Working Rules

- Do not commit or push unless the user explicitly asks in the same turn.
- Keep native behavior aligned with `../biz-suite-web/lib/device-bridge` and `../biz-suite-web/app/api/device-bridge`.
- If receipt, KOT, cash drawer, or printer payload behavior changes, check the web app payload contract too.
- F&B receipt behavior is the reference path unless the user says otherwise.
- Preserve fallback behavior: native printing should continue without optional assets such as logos if fetching/conversion fails.

## Verification

- Use Rust/Tauri checks when local `cargo` is available.
- If `cargo` or `rustfmt` is unavailable, say so clearly in the final response.
