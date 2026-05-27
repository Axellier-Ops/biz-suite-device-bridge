# Android Device Bridge

Kotlin Android bridge for tablets and LAN ESC/POS printers.

Current build:

- Fixed production Biz-Suite Cloud endpoint
- One-time pairing with locally stored device token
- Poll once or run/stop a 5-second cloud job loop while the app is open
- Receipt and KOT printing from cloud jobs
- LAN ESC/POS test print over port `9100`
- Cash drawer kick through receipt printer

Build target:

```text
Biz-Suite-Device-Bridge-Android-v0.1.2-debug.apk
```

Bluetooth printing and background/foreground-service polling are not part of
this release.
