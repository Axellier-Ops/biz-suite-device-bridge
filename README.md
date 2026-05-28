# Windows Cloud POS Shell

Tauri/Rust desktop shell for **Biz Suite Cloud POS** on Windows.

This app loads the production cloud POS directly:

```text
https://www.patas.cloud/login
```

It keeps the native Windows printer and cash drawer code from the device bridge,
but exposes it directly to the cloud POS WebView. That means the POS can print
through local Windows/LAN hardware without opening the browser print dialog and
without waiting for the cloud bridge job polling loop.

## Repository Layout

This repository contains only the Windows Cloud POS shell.

```text
src-tauri/   Native Tauri/Rust app and Windows printer commands
src/         Local fallback UI used only during shell development
scripts/     Windows build helpers
```

The older Device Bridge app remains in its own repository/release flow for
browser-based POS installs.

## Updates

Most POS changes are web changes. Because this app loads
`https://www.patas.cloud/login`, a Vercel deployment of the cloud POS is picked
up by the Windows app automatically when the app is opened, refreshed, or
navigates to the updated page.

Native shell changes still need a new Windows installer. That includes:

```text
Tauri/Rust printer commands
cash drawer commands
Windows app permissions/capabilities
startup behavior
app name, icon, version and installer settings
```

In short:

```text
Web POS UI/business logic update -> deploy patas.cloud
Native hardware/app shell update -> build and install a new EXE
```

## Native commands exposed to the cloud POS

The remote WebView is restricted to `https://www.patas.cloud` and
`https://patas.cloud` in `src-tauri/capabilities/default.json`.

The cloud POS can call:

```text
load_settings
save_settings
list_installed_printers
test_receipt_connection
print_receipt_payload
print_kot_payload
open_cash_drawer
print_sample_receipt
print_sample_kot
```

Printer settings are stored locally in the Windows application config folder as
`settings.json`, so app updates should preserve printer routes and startup
preferences.

## Web App Requirement

The cloud POS web app must include the native-detection print path:

```text
If running inside Tauri:
  call window.__TAURI__.core.invoke("print_receipt_payload", { payload })
Else:
  enqueue a device bridge print job as it does today
```

That keeps browser users on the bridge flow and gives Windows POS shell users
direct local printing. Native support exists in the app shell, but the direct
print path only works after the matching cloud POS web deployment is live.

## Run locally

Requirements:

```text
Node.js 20+
Rust stable
Microsoft WebView2 Runtime
Windows machine for installer testing
```

Commands:

```bash
npm install
npm run dev
```

## Build Windows installer

```powershell
./scripts/build-windows.ps1
```

or:

```bash
./scripts/build-windows.sh
```

The versioned installer is copied to:

```text
downloads/windows/Biz-Suite-Cloud-POS-Windows-v0.1.0.exe
```

The raw Tauri installer is generated under:

```text
src-tauri/target/release/bundle/nsis/
```

## Manual Build

```bash
npm install
npm run build
```
