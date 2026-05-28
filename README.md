# Android Cloud POS Shell

Native Android WebView shell for **Biz Suite Cloud POS**.

The app loads:

```text
https://www.patas.cloud/login
```

It exposes a native JavaScript bridge as `window.BizSuiteAndroid` so the live web
POS can print directly to local Android hardware without using the old cloud
polling bridge.

## Supported Hardware

Current Android support is for LAN/IP ESC/POS thermal printers on port `9100`.

```text
Receipt printer: LAN/IP ESC/POS
KOT printer: LAN/IP ESC/POS
Cash drawer: RJ11/RJ12 drawer connected to receipt printer
```

USB and Bluetooth Android printing are intentionally not included in this first
shell. They need separate hardware-specific work.

## Web App Requirement

The live `biz-suite-web` deployment must detect:

```text
window.BizSuiteAndroid
```

and call:

```text
printReceiptPayload(JSON.stringify(payload))
printKotPayload(JSON.stringify(payload))
openCashDrawer()
loadSettings()
saveSettings(JSON.stringify(settings))
```

Browser users keep using the normal fallback path.

## Build

Requirements:

```text
Android Studio or Android Gradle Plugin toolchain
JDK 17
Android SDK 35
```

From the repo root:

```bash
./gradlew assembleDebug
```

If using Android Studio, open this repository and build the `app` module.
