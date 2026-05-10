# Cloud Integration Contract

The Biz-Suite Cloud web app should expose endpoints or Supabase RPC/actions for the bridge.

## Pair device

```http
POST /api/device-bridge/pair
```

Request:

```json
{
  "pairingCode": "842913",
  "deviceName": "Front Counter PC",
  "platform": "windows",
  "appVersion": "0.1.0"
}
```

Response:

```json
{
  "deviceId": "uuid",
  "deviceToken": "secure-token",
  "tenantId": "uuid",
  "locationId": "uuid"
}
```

## Poll jobs

```http
POST /api/device-bridge/jobs/poll
Authorization: Bearer DEVICE_TOKEN
```

Response:

```json
{
  "jobs": []
}
```

## Complete job

```http
POST /api/device-bridge/jobs/{jobId}/complete
Authorization: Bearer DEVICE_TOKEN
```

## Fail job

```http
POST /api/device-bridge/jobs/{jobId}/fail
Authorization: Bearer DEVICE_TOKEN
```
