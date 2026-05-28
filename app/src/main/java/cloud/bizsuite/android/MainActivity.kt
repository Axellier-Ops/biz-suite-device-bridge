package cloud.bizsuite.android

import android.app.Activity
import android.os.Bundle
import android.webkit.JavascriptInterface
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import org.json.JSONArray
import org.json.JSONObject

class MainActivity : Activity() {
    private lateinit var webView: WebView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        WebView.setWebContentsDebuggingEnabled(true)
        webView = WebView(this).apply {
            webViewClient = WebViewClient()
            settings.javaScriptEnabled = true
            settings.domStorageEnabled = true
            settings.cacheMode = WebSettings.LOAD_DEFAULT
            addJavascriptInterface(AndroidBridge(), "BizSuiteAndroid")
            loadUrl("https://www.patas.cloud/login")
        }
        setContentView(webView)
    }

    override fun onBackPressed() {
        if (::webView.isInitialized && webView.canGoBack()) {
            webView.goBack()
            return
        }
        super.onBackPressed()
    }

    inner class AndroidBridge {
        private val preferences = getSharedPreferences("cloud_pos_settings", MODE_PRIVATE)
        private val printerClient = EscPosNetworkPrinterClient()

        @JavascriptInterface
        fun loadSettings(): String = settingsJson().toString()

        @JavascriptInterface
        fun saveSettings(settings: String): String {
            val json = JSONObject(settings)
            preferences.edit()
                .putString("receipt_printer_target", json.optString("receiptPrinterTarget", ""))
                .putString("kot_printer_target", json.optString("kotPrinterTarget", ""))
                .putInt("printer_port", json.optInt("printerPort", 9100))
                .apply()
            return "Android printer routing saved."
        }

        @JavascriptInterface
        fun listInstalledPrinters(): String = JSONArray().toString()

        @JavascriptInterface
        fun testReceiptConnection(settings: String): String {
            val json = JSONObject(settings)
            return printerClient.testConnection(
                json.optString("receiptPrinterTarget"),
                json.optInt("printerPort", 9100),
            )
        }

        @JavascriptInterface
        fun printSampleReceipt(settings: String): String {
            val json = JSONObject(settings)
            return printerClient.testPrint(
                json.optString("receiptPrinterTarget"),
                json.optInt("printerPort", 9100),
            )
        }

        @JavascriptInterface
        fun printSampleKot(settings: String): String {
            val json = JSONObject(settings)
            val target = json.optString("kotPrinterTarget").ifBlank {
                json.optString("receiptPrinterTarget")
            }
            return printerClient.testKot(target, json.optInt("printerPort", 9100))
        }

        @JavascriptInterface
        fun printReceiptPayload(payload: String): String {
            val settings = settingsJson()
            return printerClient.printReceipt(
                settings.optString("receiptPrinterTarget"),
                settings.optInt("printerPort", 9100),
                JSONObject(payload),
            )
        }

        @JavascriptInterface
        fun printKotPayload(payload: String): String {
            val settings = settingsJson()
            val target = settings.optString("kotPrinterTarget").ifBlank {
                settings.optString("receiptPrinterTarget")
            }
            return printerClient.printKot(
                target,
                settings.optInt("printerPort", 9100),
                JSONObject(payload),
            )
        }

        @JavascriptInterface
        fun openCashDrawer(): String {
            val settings = settingsJson()
            return printerClient.kickDrawer(
                settings.optString("receiptPrinterTarget"),
                settings.optInt("printerPort", 9100),
            )
        }

        private fun settingsJson(): JSONObject {
            val receiptTarget = preferences.getString("receipt_printer_target", "") ?: ""
            val kotTarget = preferences.getString("kot_printer_target", "") ?: ""
            val port = preferences.getInt("printer_port", 9100)
            return JSONObject()
                .put("settingsVersion", 1)
                .put("deviceToken", "")
                .put("deviceId", "")
                .put("receiptConnectionType", "lan")
                .put("receiptPrinterTarget", receiptTarget)
                .put("kotConnectionType", "lan")
                .put("kotPrinterTarget", kotTarget)
                .put("printerPort", port)
                .put("deviceName", "Android POS Tablet")
                .put("launchOnStartup", false)
        }
    }
}
