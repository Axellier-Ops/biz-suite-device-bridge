package com.bizsuite.devicebridge

import java.io.OutputStream
import java.net.InetSocketAddress
import java.net.Socket

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

    private fun connect(ip: String): Socket {
        val socket = Socket()
        socket.connect(InetSocketAddress(ip, 9100), 3000)
        return socket
    }
}
