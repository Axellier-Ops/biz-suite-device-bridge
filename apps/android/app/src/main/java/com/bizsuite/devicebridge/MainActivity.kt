package com.bizsuite.devicebridge

import android.app.Activity
import android.os.Bundle
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.TextView

class MainActivity : Activity() {
    private lateinit var status: TextView
    private lateinit var pairingCode: EditText
    private lateinit var printerIp: EditText
    private val printerClient = EscPosNetworkPrinterClient()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        status = TextView(this).apply {
            text = "Not paired"
            textSize = 18f
        }

        pairingCode = EditText(this).apply {
            hint = "Pairing code"
        }

        printerIp = EditText(this).apply {
            hint = "Printer IP e.g. 192.168.1.50"
        }

        val pairButton = Button(this).apply {
            text = "Pair"
            setOnClickListener {
                val code = pairingCode.text.toString().trim()
                status.text = if (code.isEmpty()) {
                    "Pairing code is required."
                } else {
                    // TODO: call Biz-Suite Cloud pairing endpoint and store device token.
                    "Paired with code $code"
                }
            }
        }

        val testPrintButton = Button(this).apply {
            text = "Test Print"
            setOnClickListener {
                val ip = printerIp.text.toString().trim()
                Thread {
                    val result = printerClient.testPrint(ip)
                    runOnUiThread { status.text = result }
                }.start()
            }
        }

        val drawerButton = Button(this).apply {
            text = "Open Cash Drawer"
            setOnClickListener {
                val ip = printerIp.text.toString().trim()
                Thread {
                    val result = printerClient.kickDrawer(ip)
                    runOnUiThread { status.text = result }
                }.start()
            }
        }

        val layout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(32, 32, 32, 32)
            addView(status)
            addView(pairingCode)
            addView(pairButton)
            addView(printerIp)
            addView(testPrintButton)
            addView(drawerButton)
        }

        setContentView(layout)
    }
}
