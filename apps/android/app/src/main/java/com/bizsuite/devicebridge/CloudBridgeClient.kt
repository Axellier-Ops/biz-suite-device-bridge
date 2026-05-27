package com.bizsuite.devicebridge

import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

data class PairResult(val deviceId: String, val deviceToken: String)
data class DeviceJob(val id: String, val jobType: String, val printerRole: String?, val payload: JSONObject)

class CloudBridgeClient {
    private val baseUrl = "https://www.patas.cloud/api/device-bridge"
    private val appVersion = "0.1.1"

    fun pair(code: String, deviceName: String, existingToken: String?): PairResult {
        val body = JSONObject()
            .put("pairingCode", code)
            .put("deviceName", deviceName)
            .put("platform", "android")
            .put("appVersion", appVersion)
        val response = post("/pair", body, existingToken)
        return PairResult(response.getString("deviceId"), response.getString("deviceToken"))
    }

    fun pollJobs(token: String): List<DeviceJob> {
        val response = post("/jobs/poll", JSONObject(), token)
        val jobs = response.getJSONArray("jobs")
        return (0 until jobs.length()).map { index ->
            val job = jobs.getJSONObject(index)
            DeviceJob(
                id = job.getString("id"),
                jobType = job.getString("jobType"),
                printerRole = job.optString("printerRole").ifBlank { null },
                payload = job.optJSONObject("payload") ?: JSONObject(),
            )
        }
    }

    fun completeJob(token: String, jobId: String) {
        post("/jobs/$jobId/complete", JSONObject(), token)
    }

    fun failJob(token: String, jobId: String, error: String) {
        post("/jobs/$jobId/fail", JSONObject().put("error", error), token)
    }

    private fun post(path: String, body: JSONObject, token: String?): JSONObject {
        val connection = URL("$baseUrl$path").openConnection() as HttpURLConnection
        return try {
            connection.requestMethod = "POST"
            connection.connectTimeout = 10_000
            connection.readTimeout = 10_000
            connection.doOutput = true
            connection.setRequestProperty("Content-Type", "application/json")
            if (!token.isNullOrBlank()) {
                connection.setRequestProperty("Authorization", "Bearer $token")
            }
            connection.outputStream.use { it.write(body.toString().toByteArray()) }

            val responseText = (if (connection.responseCode in 200..299) {
                connection.inputStream
            } else {
                connection.errorStream
            })?.bufferedReader()?.use { it.readText() }.orEmpty()
            val response = if (responseText.isBlank()) JSONObject() else JSONObject(responseText)
            if (connection.responseCode !in 200..299) {
                throw IllegalStateException(response.optString("error", "Cloud request failed (${connection.responseCode})."))
            }
            response
        } finally {
            connection.disconnect()
        }
    }
}
