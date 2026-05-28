package cloud.bizsuite.android

import java.io.OutputStream
import java.net.InetSocketAddress
import java.net.Socket
import org.json.JSONObject

class EscPosNetworkPrinterClient {
    private val escInit = byteArrayOf(0x1B, 0x40)
    private val escAlignCenter = byteArrayOf(0x1B, 0x61, 0x01)
    private val escAlignLeft = byteArrayOf(0x1B, 0x61, 0x00)
    private val escBoldOn = byteArrayOf(0x1B, 0x45, 0x01)
    private val escBoldOff = byteArrayOf(0x1B, 0x45, 0x00)
    private val escCut = byteArrayOf(0x1D, 0x56, 0x00)
    private val drawerKick = byteArrayOf(0x1B, 0x70, 0x00, 0x19, 0xFA.toByte())

    fun testConnection(ip: String, port: Int): String {
        if (ip.isBlank()) return "Printer IP is required."
        return try {
            connect(ip, port).use { }
            "Receipt printer route looks valid."
        } catch (e: Exception) {
            "Could not connect to printer: ${e.message}"
        }
    }

    fun testPrint(ip: String, port: Int): String {
        if (ip.isBlank()) return "Printer IP is required."

        return try {
            connect(ip, port).use { socket ->
                val out: OutputStream = socket.getOutputStream()
                out.write(escInit)
                out.write(escAlignCenter)
                out.write(escBoldOn)
                out.write("Biz Suite Cloud POS\n".toByteArray())
                out.write(escBoldOff)
                out.write("Android Test Receipt\n".toByteArray())
                out.write(escAlignLeft)
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

    fun testKot(ip: String, port: Int): String {
        if (ip.isBlank()) return "Printer IP is required."
        return printBytes(ip, port, kotBytes(sampleKotPayload()), "Sample KOT sent.")
    }

    fun kickDrawer(ip: String, port: Int): String {
        if (ip.isBlank()) return "Printer IP is required."

        return try {
            connect(ip, port).use { socket ->
                val out: OutputStream = socket.getOutputStream()
                out.write(drawerKick)
                out.flush()
            }
            "Cash drawer kick sent."
        } catch (e: Exception) {
            "Could not open cash drawer: ${e.message}"
        }
    }

    fun printReceipt(ip: String, port: Int, payload: JSONObject): String {
        return printBytes(ip, port, receiptBytes(payload), "Receipt sent.")
    }

    fun printKot(ip: String, port: Int, payload: JSONObject): String {
        return printBytes(ip, port, kotBytes(payload), "Kitchen ticket sent.")
    }

    private fun printBytes(ip: String, port: Int, bytes: ByteArray, success: String): String {
        if (ip.isBlank()) return "Printer IP is required."
        return try {
            connect(ip, port).use { socket ->
                socket.getOutputStream().use { out ->
                    out.write(bytes)
                    out.flush()
                }
            }
            success
        } catch (error: Exception) {
            "Could not print: ${error.message}"
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
                    append(quantity(item.optDouble("quantity", 0.0))).append(" x ")
                    append(item.optString("name", "Item")).append("  ")
                    append("%.2f".format(item.optDouble("total", 0.0))).append('\n')
                }
            }
            append("--------------------------------\n")
            append("Subtotal: ").append("%.2f".format(payload.optDouble("subtotal", 0.0))).append('\n')
            val discount = payload.optDouble("discount", 0.0)
            if (discount > 0.0) append("Discount: -").append("%.2f".format(discount)).append('\n')
            val service = payload.optDouble("serviceCharge", 0.0)
            if (service > 0.0) append("Service: ").append("%.2f".format(service)).append('\n')
            val tax = payload.optDouble("tax", 0.0)
            if (tax > 0.0) append("VAT: ").append("%.2f".format(tax)).append('\n')
            append("TOTAL: ").append("%.2f".format(payload.optDouble("total", 0.0))).append("\n\nThank you.\n\n\n")
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
                    append(quantity(item.optDouble("quantity", 0.0))).append(" x ")
                    append(item.optString("name", "Item")).append('\n')
                    item.optString("notes").takeIf { it.isNotBlank() }?.let { append("  Note: ").append(it).append('\n') }
                }
            }
            append("--------------------------------\n\n\n")
        }
        return text.toByteArray() + escCut
    }

    private fun sampleKotPayload() = JSONObject()
        .put("businessName", "Demo F&B")
        .put("orderNumber", "TEST-1001")
        .put("tableName", "Table 04")
        .put(
            "items",
            org.json.JSONArray()
                .put(JSONObject().put("name", "Chicken Kottu").put("quantity", 2).put("notes", "no chilli"))
                .put(JSONObject().put("name", "Lime Juice").put("quantity", 1)),
        )

    private fun quantity(value: Double): String {
        return if (value % 1.0 == 0.0) "%.0f".format(value) else "%.2f".format(value)
    }

    private fun connect(ip: String, port: Int): Socket {
        val socket = Socket()
        socket.connect(InetSocketAddress(ip, if (port > 0) port else 9100), 3000)
        return socket
    }
}
