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
apps/android   Android/Kotlin bridge for tablets and Bluetooth/network printers
packages/shared-protocol   Shared job/device protocol definitions
```

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
