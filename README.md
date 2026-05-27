# Biz-Suite Device Bridge

Local hardware bridge for the **Biz-Suite Cloud web app**.

The bridge connects Biz-Suite Cloud to local business hardware:

- Receipt printers
- KOT / kitchen printers
- Cash drawers
- Future barcode scanners, customer displays, and scales

The cloud app creates device jobs. The local bridge securely pairs with a tenant/location/register, polls or subscribes for jobs, executes them locally, and reports success/failure back to Biz-Suite Cloud.

## Apps

```text
apps/windows   Tauri/Rust desktop bridge for Windows POS counters
apps/android   Android/Kotlin bridge for tablets and LAN printers
packages/shared-protocol   Shared job/device protocol definitions
```

## Versioned build outputs

Local scripts copy generated files into versioned names:

```text
downloads/windows/Biz-Suite-Device-Bridge-Windows-v0.1.3.exe
downloads/android/Biz-Suite-Device-Bridge-Android-v0.1.2-debug.apk
```

The `downloads` folder is ignored except for `.gitkeep`, because installer/APK files should be generated locally or downloaded from GitHub Actions artifacts instead of committed to Git.

## Build Windows EXE

Run this on a Windows machine:

```powershell
./scripts/build-windows.ps1
```

Output:

```text
downloads/windows/Biz-Suite-Device-Bridge-Windows-v0.1.3.exe
```

## Build Android APK

```powershell
./scripts/build-android.ps1
```

or:

```bash
bash scripts/build-android.sh
```

Output:

```text
downloads/android/Biz-Suite-Device-Bridge-Android-v0.1.2-debug.apk
```

## Build both

```powershell
./scripts/build-all.ps1
```

or:

```bash
bash scripts/build-all.sh
```

## GitHub Actions downloads

Use the workflows from GitHub Actions:

```text
Build Windows Device Bridge
Build Android Device Bridge
```

Run the workflow manually, then download the artifact:

```text
Biz-Suite-Device-Bridge-Windows
Biz-Suite-Device-Bridge-Android
```

The artifact contents use the same versioned filenames as the local `downloads/` output.

## MVP target

Start with Windows first, then Android.

```text
V1:
- Pair device with Biz-Suite Cloud
- Register device heartbeat
- Poll print jobs
- Print receipt/KOT through ESC/POS
- Trigger cash drawer kick through receipt printer
- Mark jobs printed/failed
```
