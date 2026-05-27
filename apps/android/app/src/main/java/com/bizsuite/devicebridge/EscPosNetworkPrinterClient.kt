package com.bizsuite.devicebridge

import java.io.OutputStream
import java.net.InetSocketAddress
import java.net.Socket
import org.json.JSONObject

class EscPosNetworkPrinterClient {
    private val escInit = byteArrayOf(0x1B, 0x40)
    private val escCut = byteArrayOf(0x1D, 0x56, 0x00)
    private val drawerKick = byteArrayOf(0x1B, 0x70, 0x00, 0x19, 0xFA.toByte())

    fun testPrint(ip: String): String {
        if (ip.isBlank()) return "Printer IP is required."

        return try {
            connect(ip).use { socket ->
                val out: OutputStream = socket.getOutputStream()
                out.write(escInit)
                out.write("Biz-Suite Device Bridge\n".toByteArray())
                out.write("Android Test Print\n".toByteArray())
                out.write("--------------------------\n".toByteArray())
                out.write("Printer connected successfully.\n\n\n".toByteArray())
                out.write(escCut)
                out.flush()
            }
            "Test print sent."
        } catch (e: Exception) {
            "Could not print: ${e.message}"
        }
    }

    fun kickDrawer(ip: String): String {
        if (ip.isBlank()) return "Printer IP is required."

        return try {
            connect(ip).use { socket ->
                val out: OutputStream = socket.getOutputStream()
                out.write(drawerKick)
                out.flush()
            }
            "Cash drawer kick sent."
        } catch (e: Exception) {
            "Could not open cash drawer: ${e.message}"
        }
    }

    fun executeJob(ip: String, job: DeviceJob): String? {
        if (ip.isBlank()) return "Printer IP is required."
        val bytes = when (job.jobType) {
            "drawer_kick" -> drawerKick
            "kot_print" -> kotBytes(job.payload)
            else -> receiptBytes(job.payload)
        }
        return try {
            connect(ip).use { socket ->
                socket.getOutputStream().use { out ->
                    out.write(bytes)
                    out.flush()
                }
            }
            null
        } catch (error: Exception) {
            "Could not execute ${job.jobType}: ${error.message}"
        }
    }

    private fun receiptBytes(payload: JSONObject): ByteArray {
        val text = buildString {
            append("\u001B\u0040")
            append(payload.optString("businessName", "BIZ-SUITE CLOUD")).append('\n')
            append("Receipt\n")
            payload.optString("orderNumber").takeIf { it.isNotBlank() }?.let { append("Order: ").append(it).append('\n') }
            append("--------------------------------\n")
            val items = payload.optJSONArray("items")
            if (items != null) {
                for (index in 0 until items.length()) {
                    val item = items.getJSONObject(index)
                    append(item.optDouble("quantity", 0.0)).append(" x ")
                    append(item.optString("name", "Item")).append("  ")
                    append("%.2f".format(item.optDouble("total", 0.0))).append('\n')
                }
            }
            append("--------------------------------\n")
            append("Total: ").append("%.2f".format(payload.optDouble("total", 0.0))).append("\n\nThank you.\n\n\n")
        }
        return text.toByteArray() + escCut
    }

    private fun kotBytes(payload: JSONObject): ByteArray {
        val text = buildString {
            append("\u001B\u0040")
            append("KITCHEN ORDER TICKET\n")
            payload.optString("orderNumber").takeIf { it.isNotBlank() }?.let { append("Order: ").append(it).append('\n') }
            payload.optString("tableName").takeIf { it.isNotBlank() }?.let { append("Table: ").append(it).append('\n') }
            append("--------------------------------\n")
            val items = payload.optJSONArray("items")
            if (items != null) {
                for (index in 0 until items.length()) {
                    val item = items.getJSONObject(index)
                    append(item.optDouble("quantity", 0.0)).append(" x ")
                    append(item.optString("name", "Item")).append('\n')
                    item.optString("notes").takeIf { it.isNotBlank() }?.let { append("  Note: ").append(it).append('\n') }
                }
            }
            append("--------------------------------\n\n\n")
        }
        return text.toByteArray() + escCut
    }

    private fun connect(ip: String): Socket {
        val socket = Socket()
        socket.connect(InetSocketAddress(ip, 9100), 3000)
        return socket
    }
}
