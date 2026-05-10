# Biz-Suite Device Bridge Plan

## Product names

```text
Biz-Suite Cloud web app
Biz-Suite Device Bridge
```

## Repo decision

This repo contains the local hardware bridge apps. It is separate from the Biz-Suite Cloud web app because device printing, cash drawer control, installers, OS permissions, and local hardware access are not normal web-dashboard concerns.

## Repo layout

```text
biz-suite-device-bridge/
  apps/
    windows/
    android/
  packages/
    shared-protocol/
  docs/
  PLAN.md
  README.md
```

## Platform priority

```text
1. Windows Device Bridge
2. Android Device Bridge
3. macOS only if customers ask
4. Ubuntu/Linux only if customers ask
```

## Why one repo but two apps?

Use one repo for planning, protocol, versioning, and shared concepts. Use two app implementations because Windows and Android hardware access is different.

Windows needs:

```text
Tauri/Rust
LAN/USB printer support
startup/background process
ESC/POS printing
cash drawer kick
```

Android needs:

```text
Kotlin
Bluetooth permissions
USB permissions
LAN printer support
foreground service rules
```

## High-level architecture

```text
Biz-Suite Cloud web app
    ↓
device_jobs / print_jobs / device_registrations
    ↓
Biz-Suite Device Bridge
    ↓
Receipt printer / KOT printer / cash drawer
```

## Cloud tables needed

### device_registrations

```text
id
tenant_id
location_id
register_id
device_name
device_type: windows | android
pairing_status
last_seen_at
app_version
created_at
```

### print_devices

```text
id
tenant_id
location_id
bridge_device_id
name
printer_role: receipt | kot | bar | label
connection_type: lan | usb | bluetooth
address
port
is_default
created_at
```

### device_jobs

```text
id
tenant_id
location_id
module_key
bridge_device_id
job_type: receipt_print | kot_print | drawer_kick | test_print
payload
status: pending | processing | completed | failed
attempts
last_error
created_at
claimed_at
completed_at
```

## Pairing flow

```text
1. Owner opens Biz-Suite Cloud web app
2. Settings → Devices → Add Device Bridge
3. Cloud shows pairing code
4. User opens Biz-Suite Device Bridge
5. User enters pairing code
6. Bridge registers against tenant/location/register
7. User selects receipt printer and KOT printer
8. User sends test print
9. Device is ready
```

## MVP scope

Must have:

```text
Pairing screen
Device status screen
LAN printer test print
Receipt print
KOT print
Cash drawer kick through receipt printer
Job status update
```

Not MVP:

```text
macOS
Ubuntu
customer display
barcode scanner
scales
advanced kitchen routing
auto-updater
```
