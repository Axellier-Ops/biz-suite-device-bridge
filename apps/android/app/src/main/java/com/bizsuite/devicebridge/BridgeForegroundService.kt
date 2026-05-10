package com.bizsuite.devicebridge

import android.app.Service
import android.content.Intent
import android.os.IBinder

class BridgeForegroundService : Service() {
    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        // TODO:
        // 1. Load stored device token
        // 2. Start heartbeat to Biz-Suite Cloud
        // 3. Poll or subscribe for device jobs
        // 4. Execute print/cash drawer jobs
        // 5. Report job results
        return START_STICKY
    }
}
