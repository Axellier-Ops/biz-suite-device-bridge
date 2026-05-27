package com.bizsuite.devicebridge

import android.app.Activity
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.text.InputType
import android.view.View
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.TextView

class MainActivity : Activity() {
    private val preferences by lazy { getSharedPreferences("device_bridge_settings", MODE_PRIVATE) }
    private val cloudClient = CloudBridgeClient()
    private val printerClient = EscPosNetworkPrinterClient()
    private val mainHandler = Handler(Looper.getMainLooper())

    private lateinit var status: TextView
    private lateinit var pairingCode: EditText
    private lateinit var deviceName: EditText
    private lateinit var printerIp: EditText
    private lateinit var pairButton: Button
    private lateinit var pairAnotherButton: Button
    private lateinit var pollOnceButton: Button
    private lateinit var pollLoopButton: Button
    private var polling = false
    private var pollInProgress = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        status = TextView(this).apply {
            textSize = 18f
        }

        pairingCode = EditText(this).apply {
            hint = "One-time pairing code"
            inputType = InputType.TYPE_CLASS_NUMBER
        }

        deviceName = EditText(this).apply {
            hint = "Device name"
            setText(preferences.getString("device_name", "Android Counter Tablet"))
        }

        printerIp = EditText(this).apply {
            hint = "Receipt / KOT printer IP e.g. 192.168.1.50"
            inputType = InputType.TYPE_CLASS_PHONE
            setText(preferences.getString("printer_ip", ""))
        }

        pairButton = Button(this).apply {
            text = "Pair with cloud"
            setOnClickListener { pairWithCloud() }
        }

        pairAnotherButton = Button(this).apply {
            text = "Pair another device"
            setOnClickListener { setPairingState(true, true) }
        }

        val savePrinterButton = Button(this).apply {
            text = "Save printer settings"
            setOnClickListener {
                preferences.edit().putString("printer_ip", printerIp.text.toString().trim()).apply()
                status.text = "Printer settings saved."
            }
        }

        val testPrintButton = Button(this).apply {
            text = "Print sample receipt"
            setOnClickListener {
                runInBackground({ printerClient.testPrint(printerIp.text.toString().trim()) })
            }
        }

        val drawerButton = Button(this).apply {
            text = "Open cash drawer"
            setOnClickListener {
                runInBackground({ printerClient.kickDrawer(printerIp.text.toString().trim()) })
            }
        }

        pollOnceButton = Button(this).apply {
            text = "Poll jobs once"
            setOnClickListener { pollJobsOnce() }
        }

        pollLoopButton = Button(this).apply {
            text = "Run polling loop"
            setOnClickListener { togglePollingLoop() }
        }

        val layout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(32, 32, 32, 32)
            addView(status)
            addView(pairingCode)
            addView(deviceName)
            addView(pairButton)
            addView(pairAnotherButton)
            addView(printerIp)
            addView(savePrinterButton)
            addView(testPrintButton)
            addView(drawerButton)
            addView(pollOnceButton)
            addView(pollLoopButton)
        }

        setContentView(layout)
        setPairingState(storedToken().isNotBlank())
    }

    private fun setPairingState(isPaired: Boolean, editing: Boolean = false) {
        val locked = isPaired && !editing
        pairingCode.setText("")
        pairingCode.isEnabled = !locked
        deviceName.isEnabled = !locked
        pairButton.visibility = if (locked) View.GONE else View.VISIBLE
        pairButton.text = if (isPaired) "Apply new pairing" else "Pair with cloud"
        pairAnotherButton.visibility = if (locked) View.VISIBLE else View.GONE
        pollOnceButton.isEnabled = isPaired
        pollLoopButton.isEnabled = isPaired
        status.text = when {
            editing -> "Paired - enter a new code to change pairing."
            isPaired -> "Paired with Biz-Suite Cloud."
            else -> "Not paired."
        }
    }

    private fun pairWithCloud() {
        val code = pairingCode.text.toString().trim()
        val name = deviceName.text.toString().trim().ifBlank { "Android Counter Tablet" }
        if (code.isBlank()) {
            status.text = "Pairing code is required."
            return
        }
        runInBackground({
            val result = cloudClient.pair(code, name, storedToken().ifBlank { null })
            preferences.edit()
                .putString("device_token", result.deviceToken)
                .putString("device_id", result.deviceId)
                .putString("device_name", name)
                .apply()
            mainHandler.post { setPairingState(true) }
            "Device paired as $name."
        })
    }

    private fun pollJobsOnce() {
        if (pollInProgress) return
        val token = storedToken()
        if (token.isBlank()) {
            status.text = "Device is not paired."
            return
        }
        val ip = printerIp.text.toString().trim()
        pollInProgress = true
        runInBackground({
            val jobs = cloudClient.pollJobs(token)
            var failed = 0
            jobs.forEach { job ->
                val printError = printerClient.executeJob(ip, job)
                if (printError == null) {
                    cloudClient.completeJob(token, job.id)
                } else {
                    cloudClient.failJob(token, job.id, printError)
                    failed += 1
                }
            }
            if (failed > 0) {
                "Processed ${jobs.size} job(s); $failed failed. Check printer IP."
            } else {
                "Polled cloud and processed ${jobs.size} job(s)."
            }
        }, onFinished = { pollInProgress = false })
    }

    private fun togglePollingLoop() {
        if (polling) {
            polling = false
            pollLoopButton.text = "Run polling loop"
            status.text = "Polling stopped."
            return
        }
        polling = true
        pollLoopButton.text = "Stop polling loop"
        status.text = "Polling cloud every 5 seconds."
        pollLoop()
    }

    private fun pollLoop() {
        if (!polling) return
        pollJobsOnce()
        mainHandler.postDelayed({ pollLoop() }, 5_000)
    }

    private fun storedToken() = preferences.getString("device_token", "") ?: ""

    private fun runInBackground(action: () -> String, onFinished: () -> Unit = {}) {
        Thread {
            val result = try {
                action()
            } catch (error: Exception) {
                error.message ?: "Unexpected bridge error."
            }
            mainHandler.post {
                onFinished()
                status.text = result
            }
        }.start()
    }
}
