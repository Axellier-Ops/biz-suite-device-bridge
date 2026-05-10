# Windows Device Bridge

Tauri/Rust desktop bridge for Windows POS counters.

## What works in this testable build

```text
- Save receipt printer IP, KOT printer IP and ESC/POS port
- Test connection to a LAN printer
- Print sample receipt
- Print sample KOT
- Open cash drawer through the receipt printer
- Keep local settings in the Windows app config folder
```

## Hardware supported now

```text
LAN/Ethernet ESC/POS thermal printers on port 9100
Cash drawer connected to the receipt printer using RJ11/RJ12 drawer cable
```

USB and Bluetooth printer support are not in this first Windows test build.

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

## Testing steps

```text
1. Make sure the thermal printer is connected to the same network.
2. Print the printer network configuration page and find its IP address.
3. Open Biz-Suite Device Bridge.
4. Enter receipt printer IP.
5. Enter KOT printer IP or reuse receipt printer IP.
6. Keep port as 9100 unless the printer uses another port.
7. Click Save settings.
8. Click Test receipt connection.
9. Click Print sample receipt.
10. Click Print sample KOT.
11. If the cash drawer is plugged into the receipt printer, click Open cash drawer.
```

## Notes

The cloud pairing button is intentionally a placeholder until the Biz-Suite Cloud web app exposes the pairing and device job endpoints.
