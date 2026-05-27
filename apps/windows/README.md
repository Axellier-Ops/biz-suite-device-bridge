# Windows Device Bridge

Tauri/Rust desktop bridge for Windows POS counters.

## What works in this Windows build

```text
- Connect to the production Biz-Suite Cloud bridge endpoint
- Pair with Biz-Suite Cloud using a one-time code
- Store returned device token locally
- Poll cloud jobs manually or in a 5-second loop
- Start polling automatically when the app opens and the device is already paired
- Report job complete/fail back to Biz-Suite Cloud
- Route receipt jobs to receipt printer
- Route KOT jobs to KOT printer
- Route drawer kick jobs to receipt printer
- List Windows-installed printers
- Auto-select a single detected Windows-installed thermal printer for receipt and KOT routing
- Print to LAN/Ethernet ESC/POS printers on port 9100
- Print raw ESC/POS to Windows-installed printers
- Support USB printers after they are installed in Windows as printers
- Support Bluetooth printers after they are paired and installed in Windows as printers
- Print sample receipt
- Print sample KOT
- Open cash drawer through receipt printer
- Keep local settings in the Windows app config folder
- Preserve pairing and printer routing settings when a newer installer is installed
- Enable start-with-Windows on first run, with a preference toggle in the app
- Check Biz-Suite Cloud for a published newer installer and show a download prompt
```

## Hardware supported

### Best supported

```text
LAN/Ethernet ESC/POS thermal printers on port 9100
Cash drawer connected to receipt printer using RJ11/RJ12 drawer cable
```

### Supported through Windows printer installation

```text
USB thermal printers installed in Windows
Bluetooth thermal printers paired and installed in Windows
```

For USB/Bluetooth, install the printer driver first, then select the Windows printer name inside the Device Bridge.
When one thermal printer is detected and no printer is configured yet, the bridge selects it automatically for receipt and KOT output. If several candidate printers are installed, select the correct role manually.

## Updates and saved settings

The bridge stores its pairing token, device identity, printer routes and startup
preference in its Windows application configuration directory as `settings.json`.
Installing a newer Device Bridge version keeps this file and migrates older
settings schemas in place.

The app checks `https://www.patas.cloud/api/device-bridge/releases/windows` for
a published newer version at startup and when the operator selects **Check for
updates**. The cloud application supplies a secure installer URL through these
deployment variables:

```text
DEVICE_BRIDGE_WINDOWS_LATEST_VERSION
DEVICE_BRIDGE_WINDOWS_DOWNLOAD_URL
DEVICE_BRIDGE_WINDOWS_RELEASE_NOTES
```

This release uses an operator-confirmed download prompt. Fully in-app automatic
installation requires signed Tauri updater artifacts and a public updater feed.

## Cloud bridge endpoint

The Windows bridge connects to the production endpoint internally:

```text
https://www.patas.cloud/api/device-bridge
```

The endpoint exposes:

```text
POST /pair
POST /jobs/poll
POST /jobs/{jobId}/complete
POST /jobs/{jobId}/fail
```

## Run locally

Requirements:

```text
Node.js 20+
Rust stable
Microsoft WebView2 Runtime
Windows machine for installer testing
```

Supported deployment targets are Windows 10 and Windows 11. Current Microsoft
Edge WebView2 runtime installers are not a viable target for Windows 7/8-era
machines.

Commands:

```bash
cd apps/windows
npm install
npm run dev
```

## Build Windows installer

```bash
cd apps/windows
npm install
npm run build
```

The installer should be generated under:

```text
apps/windows/src-tauri/target/release/bundle/nsis/
```

The installer embeds the offline Microsoft Edge WebView2 Runtime installer so
setup can complete on Windows 10/11 POS machines even when WebView2 is missing
or the machine is offline during installation. This intentionally makes the
installer much larger than the previous online-bootstrapper build.

## Testing LAN printer

```text
1. Make sure the thermal printer is connected to the same network.
2. Print the printer network configuration page and find its IP address.
3. Choose LAN/IP printer for receipt and/or KOT.
4. Enter printer IP.
5. Keep port as 9100 unless the printer uses another port.
6. Click Save settings.
7. Click Test receipt connection.
8. Click Print sample receipt.
9. Click Print sample KOT.
10. If cash drawer is plugged into receipt printer, click Open cash drawer.
```

## Testing USB or Bluetooth printer

```text
1. Install/pair the printer in Windows first.
2. Confirm it appears in Windows Settings → Printers & scanners.
3. Open Biz-Suite Device Bridge.
4. If exactly one thermal printer is available, confirm it has been selected automatically.
5. If several printers appear, select the installed printer and click Use selected as receipt or Use selected as KOT.
6. Save settings.
7. Print sample receipt or sample KOT.
```

## Notes

USB/Bluetooth support uses the Windows print spooler and sends raw ESC/POS data to the installed printer. Some printer drivers may alter raw data. For reliable POS hardware, LAN/IP ESC/POS printers are still the preferred deployment.
