package cloud.bizsuite.android

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Color
import java.io.ByteArrayOutputStream
import java.io.OutputStream
import java.net.HttpURLConnection
import java.net.InetSocketAddress
import java.net.Socket
import java.net.URL
import org.json.JSONObject
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToInt

class EscPosNetworkPrinterClient {
    private val escInit = byteArrayOf(0x1B, 0x40)
    private val escAlignCenter = byteArrayOf(0x1B, 0x61, 0x01)
    private val escAlignLeft = byteArrayOf(0x1B, 0x61, 0x00)
    private val escBoldOn = byteArrayOf(0x1B, 0x45, 0x01)
    private val escBoldOff = byteArrayOf(0x1B, 0x45, 0x00)
    private val escCut = byteArrayOf(0x1D, 0x56, 0x00)
    private val drawerKick = byteArrayOf(0x1B, 0x70, 0x00, 0x19, 0xFA.toByte())
    private val maxLogoBytes = 1_000_000
    private val receiptLogoMaxWidth = 192
    private val receiptLogoMaxHeight = 96
    private val defaultReceiptLogoUrl = "https://www.patas.cloud/logo-no-bg.png"

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
        val out = ByteArrayOutputStream()
        out.write(escInit)
        out.write(escAlignCenter)
        receiptLogoBytes(payload)?.let { out.write(it) }

        val text = buildString {
            append(payload.optString("businessName", "BIZ-SUITE CLOUD")).append('\n')
            append("Receipt\n")
            append("\u001B\u0061\u0000")
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
            append("TOTAL: ").append("%.2f".format(payload.optDouble("total", 0.0))).append('\n')
            append("\u001B\u0061\u0001")
            append("\nThank you.\n")
            append("Powered by Biz Suite Cloud POS\n")
            append("\u001B\u0045\u0001")
            append("www.patas.cloud\n\n\n")
            append("\u001B\u0045\u0000")
        }
        out.write(text.toByteArray())
        out.write(escCut)
        return out.toByteArray()
    }

    private fun receiptLogoBytes(payload: JSONObject): ByteArray? {
        val logoUrl = payload.optString("logoUrl").trim().ifBlank { defaultReceiptLogoUrl }
            .takeIf { it.startsWith("https://") } ?: return null
        return try {
            val connection = (URL(logoUrl).openConnection() as HttpURLConnection).apply {
                connectTimeout = 3000
                readTimeout = 3000
            }
            connection.inputStream.use { input ->
                if (connection.contentLengthLong > maxLogoBytes) return null
                val bitmap = BitmapFactory.decodeStream(input) ?: return null
                val scale = min(
                    receiptLogoMaxWidth.toFloat() / max(bitmap.width, 1).toFloat(),
                    receiptLogoMaxHeight.toFloat() / max(bitmap.height, 1).toFloat(),
                ).coerceAtMost(1f)
                val width = max((bitmap.width * scale).roundToInt(), 1)
                val height = max((bitmap.height * scale).roundToInt(), 1)
                val resized = Bitmap.createScaledBitmap(bitmap, width, height, true)
                val rasterWidthBytes = (width + 7) / 8
                val raster = ByteArray(rasterWidthBytes * height)

                for (y in 0 until height) {
                    for (x in 0 until width) {
                        val color = resized.getPixel(x, y)
                        val alpha = Color.alpha(color).toFloat() / 255f
                        val luminance =
                            0.299f * Color.red(color) + 0.587f * Color.green(color) + 0.114f * Color.blue(color)
                        val composited = 255f * (1f - alpha) + luminance * alpha
                        if (composited < 190f) {
                            val index = y * rasterWidthBytes + x / 8
                            raster[index] = (raster[index].toInt() or (0x80 ushr (x % 8))).toByte()
                        }
                    }
                }

                ByteArrayOutputStream().apply {
                    write(escAlignCenter)
                    write(byteArrayOf(0x1D, 0x76, 0x30, 0x00))
                    write(rasterWidthBytes and 0xFF)
                    write((rasterWidthBytes ushr 8) and 0xFF)
                    write(height and 0xFF)
                    write((height ushr 8) and 0xFF)
                    write(raster)
                    write('\n'.code)
                }.toByteArray()
            }
        } catch (_: Exception) {
            null
        }
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
