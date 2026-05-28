#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::net::SocketAddr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_opener::OpenerExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::timeout;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Graphics::Printing::{
    ClosePrinter, DOC_INFO_1W, EndDocPrinter, EndPagePrinter, EnumPrintersW, OpenPrinterW,
    StartDocPrinterW, StartPagePrinter, WritePrinter, PRINTER_ENUM_CONNECTIONS, PRINTER_ENUM_LOCAL,
    PRINTER_INFO_4W,
};

const ESC_INIT: &[u8] = b"\x1B\x40";
const ESC_ALIGN_CENTER: &[u8] = b"\x1B\x61\x01";
const ESC_ALIGN_LEFT: &[u8] = b"\x1B\x61\x00";
const ESC_BOLD_ON: &[u8] = b"\x1B\x45\x01";
const ESC_BOLD_OFF: &[u8] = b"\x1B\x45\x00";
const ESC_CUT: &[u8] = b"\x1D\x56\x00";
const DRAWER_KICK: &[u8] = b"\x1B\x70\x00\x19\xFA";
const CLOUD_BRIDGE_URL: &str = "https://www.patas.cloud/api/device-bridge";
const SETTINGS_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum ConnectionType {
    Lan,
    Windows,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
struct BridgeSettings {
    settings_version: u32,
    device_token: String,
    device_id: String,
    receipt_connection_type: ConnectionType,
    receipt_printer_target: String,
    kot_connection_type: ConnectionType,
    kot_printer_target: String,
    printer_port: u16,
    device_name: String,
    launch_on_startup: bool,
}

impl Default for BridgeSettings {
    fn default() -> Self {
        Self {
            settings_version: SETTINGS_VERSION,
            device_token: String::new(),
            device_id: String::new(),
            receipt_connection_type: ConnectionType::Lan,
            receipt_printer_target: String::new(),
            kot_connection_type: ConnectionType::Lan,
            kot_printer_target: String::new(),
            printer_port: 9100,
            device_name: "Front Counter PC".to_string(),
            launch_on_startup: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairRequest {
    pairing_code: String,
    device_name: String,
    platform: String,
    app_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairResponse {
    device_id: String,
    device_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DeviceJob {
    id: String,
    job_type: String,
    printer_role: Option<String>,
    payload: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PollResponse {
    jobs: Vec<DeviceJob>,
}

#[derive(Debug, Deserialize)]
struct BridgeApiError {
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCheckResponse {
    update_available: bool,
    version: Option<String>,
    download_url: Option<String>,
    notes: Option<String>,
}

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

fn ptr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe {
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len))
    }
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| format!("Could not locate app config directory: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| format!("Could not create app config directory: {e}"))?;
    Ok(dir.join("settings.json"))
}

fn load_settings_from_disk(app: &tauri::AppHandle) -> Result<BridgeSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(BridgeSettings::default());
    }
    let content = fs::read_to_string(path).map_err(|e| format!("Could not read settings: {e}"))?;
    let mut settings: BridgeSettings =
        serde_json::from_str(&content).map_err(|e| format!("Settings file is invalid: {e}"))?;
    if settings.settings_version < SETTINGS_VERSION {
        settings.settings_version = SETTINGS_VERSION;
        save_settings_to_disk(app, &settings)?;
    }
    Ok(settings)
}

fn save_settings_to_disk(app: &tauri::AppHandle, settings: &BridgeSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let content = serde_json::to_string_pretty(settings).map_err(|e| format!("Could not encode settings: {e}"))?;
    fs::write(path, content).map_err(|e| format!("Could not save settings: {e}"))
}

fn apply_launch_on_startup(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    let is_enabled = manager
        .is_enabled()
        .map_err(|e| format!("Could not check start-with-Windows setting: {e}"))?;
    if enabled && !is_enabled {
        manager
            .enable()
            .map_err(|e| format!("Could not enable start-with-Windows: {e}"))?;
    } else if !enabled && is_enabled {
        manager
            .disable()
            .map_err(|e| format!("Could not disable start-with-Windows: {e}"))?;
    }
    Ok(())
}

async fn read_bridge_response<T: DeserializeOwned>(response: reqwest::Response, fallback: &str) -> Result<T, String> {
    let status = response.status();
    if !status.is_success() {
        let message = response.json::<BridgeApiError>().await.ok()
            .and_then(|body| body.error)
            .unwrap_or_else(|| format!("{fallback} ({status})."));
        return Err(message);
    }
    response.json::<T>().await.map_err(|e| format!("{fallback}: invalid response ({e})"))
}

async fn require_bridge_success(response: reqwest::Response, fallback: &str) -> Result<(), String> {
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    let message = response.json::<BridgeApiError>().await.ok()
        .and_then(|body| body.error)
        .unwrap_or_else(|| format!("{fallback} ({status})."));
    Err(message)
}

async fn connect_lan_printer(target: &str, port: u16) -> Result<TcpStream, String> {
    if target.trim().is_empty() { return Err("Printer target is required.".to_string()); }
    if port == 0 { return Err("Printer port is invalid.".to_string()); }
    let address: SocketAddr = format!("{}:{}", target.trim(), port)
        .parse()
        .map_err(|_| "Printer address is invalid. Use an IP address like 192.168.1.50.".to_string())?;
    timeout(Duration::from_secs(4), TcpStream::connect(address)).await
        .map_err(|_| "Printer connection timed out. Check IP, power and network.".to_string())?
        .map_err(|e| format!("Could not connect to printer: {e}"))
}

async fn send_lan(target: &str, port: u16, bytes: &[u8]) -> Result<(), String> {
    let mut stream = connect_lan_printer(target, port).await?;
    stream.write_all(bytes).await.map_err(|e| format!("Could not write to printer: {e}"))?;
    let _ = stream.shutdown().await;
    Ok(())
}

fn send_windows_printer(printer_name: &str, bytes: &[u8]) -> Result<(), String> {
    if printer_name.trim().is_empty() { return Err("Windows printer name is required.".to_string()); }
    let mut handle: HANDLE = ptr::null_mut();
    let printer_name_w = wide(printer_name);
    let doc_name_w = wide("Biz Suite Cloud POS Job");
    let raw_w = wide("RAW");

    unsafe {
        if OpenPrinterW(printer_name_w.as_ptr(), &mut handle, ptr::null_mut()) == 0 {
            return Err(format!("Could not open Windows printer '{printer_name}'. Check it is installed and online."));
        }
        let doc = DOC_INFO_1W {
            pDocName: doc_name_w.as_ptr() as *mut u16,
            pOutputFile: ptr::null_mut(),
            pDatatype: raw_w.as_ptr() as *mut u16,
        };
        if StartDocPrinterW(handle, 1, &doc as *const DOC_INFO_1W) == 0 {
            ClosePrinter(handle);
            return Err("Could not start printer document.".to_string());
        }
        if StartPagePrinter(handle) == 0 {
            EndDocPrinter(handle); ClosePrinter(handle);
            return Err("Could not start printer page.".to_string());
        }
        let mut written: u32 = 0;
        if WritePrinter(handle, bytes.as_ptr() as *const _, bytes.len() as u32, &mut written) == 0 {
            EndPagePrinter(handle); EndDocPrinter(handle); ClosePrinter(handle);
            return Err("Could not write raw ESC/POS data to Windows printer.".to_string());
        }
        EndPagePrinter(handle);
        EndDocPrinter(handle);
        ClosePrinter(handle);
    }
    Ok(())
}

fn validate_windows_printer(printer_name: &str) -> Result<(), String> {
    if printer_name.trim().is_empty() { return Err("Windows printer name is required.".to_string()); }
    let mut handle: HANDLE = ptr::null_mut();
    let printer_name_w = wide(printer_name);
    unsafe {
        if OpenPrinterW(printer_name_w.as_ptr(), &mut handle, ptr::null_mut()) == 0 {
            return Err(format!("Could not open Windows printer '{printer_name}'. Check it is installed and online."));
        }
        ClosePrinter(handle);
    }
    Ok(())
}

async fn send_to_target(connection_type: &ConnectionType, target: &str, port: u16, bytes: &[u8]) -> Result<(), String> {
    match connection_type {
        ConnectionType::Lan => send_lan(target, port, bytes).await,
        ConnectionType::Windows => send_windows_printer(target, bytes),
    }
}

fn line(label: &str, value: &str) -> Vec<u8> { format!("{:<18}{:>14}\n", label, value).into_bytes() }

fn payload_string(payload: &Value, key: &str) -> Option<String> {
    payload.get(key).and_then(|v| v.as_str()).map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn payload_number(payload: &Value, key: &str) -> f64 {
    payload.get(key)
        .and_then(|value| value.as_f64().or_else(|| value.as_str().and_then(|v| v.parse::<f64>().ok())))
        .unwrap_or(0.0)
}

fn clean_text(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_string()
}

fn truncate_chars(value: &str, max: usize) -> String {
    let mut output = String::new();
    for ch in value.chars().take(max) {
        output.push(ch);
    }
    output
}

fn money(value: f64) -> String {
    format!("{value:.2}")
}

fn currency(value: f64) -> String {
    format!("LKR {}", money(value))
}

fn quantity(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        money(value)
    }
}

fn item_quantity(item: &Value) -> f64 {
    payload_number(item, "quantity")
}

fn receipt_bytes_from_payload(payload: &Value) -> Vec<u8> {
    let mut b = Vec::new();
    let business_name = payload_string(payload, "businessName").unwrap_or_else(|| "BIZ-SUITE CLOUD".to_string());
    b.extend_from_slice(ESC_INIT);
    b.extend_from_slice(ESC_ALIGN_CENTER);
    b.extend_from_slice(ESC_BOLD_ON);
    b.extend_from_slice(clean_text(&business_name).as_bytes());
    b.extend_from_slice(b"\n");
    b.extend_from_slice(ESC_BOLD_OFF);

    for key in ["address", "phone"] {
        if let Some(value) = payload_string(payload, key) {
            b.extend_from_slice(clean_text(&value).as_bytes());
            b.extend_from_slice(b"\n");
        }
    }

    b.extend_from_slice(b"\n");
    b.extend_from_slice(ESC_ALIGN_LEFT);
    if let Some(order_number) = payload_string(payload, "orderNumber") {
        b.extend_from_slice(format!("Order: {}\n", clean_text(&order_number)).as_bytes());
    }
    if let Some(table_name) = payload_string(payload, "tableName") {
        b.extend_from_slice(format!("Table: {}\n", clean_text(&table_name)).as_bytes());
    }
    if let Some(customer_name) = payload_string(payload, "customerName") {
        b.extend_from_slice(format!("Customer: {}\n", clean_text(&customer_name)).as_bytes());
    }
    b.extend_from_slice(b"--------------------------------\n");

    if let Some(items) = payload.get("items").and_then(|items| items.as_array()) {
        for item in items {
            let name = payload_string(item, "name").unwrap_or_else(|| "Item".to_string());
            let item_quantity = item_quantity(item);
            let total = payload_number(item, "total");
            let label = truncate_chars(&format!("{}x {}", quantity(item_quantity), clean_text(&name)), 21);
            b.extend_from_slice(format!("{:<21}{:>11}\n", label, currency(total)).as_bytes());
            if let Some(notes) = payload_string(item, "notes") {
                b.extend_from_slice(format!("  Note: {}\n", truncate_chars(&clean_text(&notes), 28)).as_bytes());
            }
        }
    }

    b.extend_from_slice(b"--------------------------------\n");
    b.extend(line("Subtotal", &currency(payload_number(payload, "subtotal"))).as_slice());
    let discount = payload_number(payload, "discount");
    if discount > 0.0 {
        b.extend(line("Discount", &format!("-{}", currency(discount))).as_slice());
    }
    let service_charge = payload_number(payload, "serviceCharge");
    if service_charge > 0.0 {
        b.extend(line("Service", &currency(service_charge)).as_slice());
    }
    let tax = payload_number(payload, "tax");
    if tax > 0.0 {
        b.extend(line("VAT (15%)", &currency(tax)).as_slice());
    }
    b.extend_from_slice(ESC_BOLD_ON);
    b.extend(line("TOTAL", &currency(payload_number(payload, "total"))).as_slice());
    b.extend_from_slice(ESC_BOLD_OFF);

    if let Some(payment_method) = payload_string(payload, "paymentMethod") {
        b.extend_from_slice(format!("\nPayment: {}\n", clean_text(&payment_method).to_uppercase()).as_bytes());
    }
    b.extend_from_slice(b"\nThank you.\n\n\n");
    b.extend_from_slice(ESC_CUT);
    b
}

fn kot_bytes_from_payload(payload: &Value) -> Vec<u8> {
    let mut b = Vec::new();
    let business_name = payload_string(payload, "businessName").unwrap_or_else(|| "BIZ-SUITE CLOUD".to_string());
    b.extend_from_slice(ESC_INIT);
    b.extend_from_slice(ESC_ALIGN_CENTER);
    b.extend_from_slice(ESC_BOLD_ON);
    b.extend_from_slice(b"KITCHEN ORDER TICKET\n");
    b.extend_from_slice(ESC_BOLD_OFF);
    b.extend_from_slice(clean_text(&business_name).as_bytes());
    b.extend_from_slice(b"\n");

    if let Some(order_number) = payload_string(payload, "orderNumber") {
        b.extend_from_slice(format!("Order {}\n", clean_text(&order_number)).as_bytes());
    }
    if let Some(table_name) = payload_string(payload, "tableName") {
        b.extend_from_slice(format!("{}\n", clean_text(&table_name)).as_bytes());
    }

    b.extend_from_slice(b"\n");
    b.extend_from_slice(ESC_ALIGN_LEFT);
    b.extend_from_slice(b"--------------------------------\n");

    if let Some(items) = payload.get("items").and_then(|items| items.as_array()) {
        for item in items {
            let name = payload_string(item, "name").unwrap_or_else(|| "Item".to_string());
            b.extend_from_slice(ESC_BOLD_ON);
            b.extend_from_slice(format!("{} x {}\n", money(item_quantity(item)), clean_text(&name)).as_bytes());
            b.extend_from_slice(ESC_BOLD_OFF);
            if let Some(notes) = payload_string(item, "notes") {
                b.extend_from_slice(format!("  Note: {}\n", clean_text(&notes)).as_bytes());
            }
            if let Some(modifiers) = item.get("modifiers").and_then(|value| value.as_object()) {
                for (key, value) in modifiers {
                    if let Some(modifier) = value.as_str() {
                        b.extend_from_slice(format!("  {}: {}\n", clean_text(key), clean_text(modifier)).as_bytes());
                    }
                }
            }
        }
    }

    b.extend_from_slice(b"--------------------------------\nPrinted by Biz Suite Cloud POS\n\n\n");
    b.extend_from_slice(ESC_CUT);
    b
}

fn sample_receipt_bytes() -> Vec<u8> {
    receipt_bytes_from_payload(&serde_json::json!({
        "businessName": "Demo F&B",
        "address": "123 Demo Street",
        "phone": "+94 11 234 5678",
        "orderNumber": "TEST-1001",
        "tableName": "Table 04",
        "items": [
            { "name": "Chicken Kottu", "quantity": 2, "total": 2400.00 },
            { "name": "Lime Juice", "quantity": 1, "total": 550.00 }
        ],
        "subtotal": 2950.00,
        "discount": 0.00,
        "serviceCharge": 295.00,
        "tax": 486.75,
        "total": 3731.75,
        "paymentMethod": "cash"
    }))
}

fn sample_kot_bytes() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(ESC_INIT); b.extend_from_slice(ESC_ALIGN_CENTER); b.extend_from_slice(ESC_BOLD_ON);
    b.extend_from_slice(b"KITCHEN ORDER TICKET\n"); b.extend_from_slice(ESC_BOLD_OFF);
    b.extend_from_slice(b"Table 04 | TEST-1001\n\n"); b.extend_from_slice(ESC_ALIGN_LEFT);
    b.extend_from_slice(b"--------------------------------\n"); b.extend_from_slice(ESC_BOLD_ON); b.extend_from_slice(b"2 x Chicken Kottu\n"); b.extend_from_slice(ESC_BOLD_OFF);
    b.extend_from_slice(b"    Note: no chilli\n"); b.extend_from_slice(ESC_BOLD_ON); b.extend_from_slice(b"1 x Lime Juice\n"); b.extend_from_slice(ESC_BOLD_OFF);
    b.extend_from_slice(b"--------------------------------\nPrinted by Biz Suite Cloud POS\n\n\n"); b.extend_from_slice(ESC_CUT); b
}

fn bytes_from_job(job: &DeviceJob) -> Vec<u8> {
    match job.job_type.as_str() {
        "kot_print" => job.payload.as_ref().map(kot_bytes_from_payload).unwrap_or_else(sample_kot_bytes),
        "drawer_kick" => DRAWER_KICK.to_vec(),
        "receipt_print" => job.payload.as_ref().map(receipt_bytes_from_payload).unwrap_or_else(sample_receipt_bytes),
        _ => sample_receipt_bytes(),
    }
}

async fn execute_job(settings: &BridgeSettings, job: &DeviceJob) -> Result<(), String> {
    let bytes = bytes_from_job(job);
    if job.job_type == "drawer_kick" || job.job_type == "test_print" || job.printer_role.as_deref() == Some("receipt") || job.job_type == "receipt_print" {
        send_to_target(&settings.receipt_connection_type, &settings.receipt_printer_target, settings.printer_port, &bytes).await
    } else {
        if settings.kot_printer_target.trim().is_empty() {
            send_to_target(&settings.receipt_connection_type, &settings.receipt_printer_target, settings.printer_port, &bytes).await
        } else {
            send_to_target(&settings.kot_connection_type, &settings.kot_printer_target, settings.printer_port, &bytes).await
        }
    }
}

async fn post_job_status(settings: &BridgeSettings, job_id: &str, status: &str, error: Option<String>) -> Result<(), String> {
    let token = settings.device_token.trim();
    if token.is_empty() { return Ok(()); }
    let url = format!("{CLOUD_BRIDGE_URL}/jobs/{job_id}/{status}");
    let body = serde_json::json!({ "error": error });
    let response = reqwest::Client::new().post(url).bearer_auth(token).json(&body).send().await
        .map_err(|e| format!("Could not report job status: {e}"))?;
    require_bridge_success(response, "Could not report job status").await
}

#[tauri::command]
async fn load_settings(app: tauri::AppHandle) -> Result<BridgeSettings, String> { load_settings_from_disk(&app) }

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, mut settings: BridgeSettings) -> Result<String, String> {
    let existing = load_settings_from_disk(&app).unwrap_or_default();
    settings.settings_version = SETTINGS_VERSION;
    settings.device_token = existing.device_token;
    settings.device_id = existing.device_id;
    if !settings.device_token.trim().is_empty() {
        settings.device_name = existing.device_name;
    }
    if settings.printer_port == 0 { return Err("Printer port is invalid.".to_string()); }
    save_settings_to_disk(&app, &settings)?;
    apply_launch_on_startup(&app, settings.launch_on_startup)?;
    Ok("Settings saved.".to_string())
}

#[tauri::command]
async fn check_for_updates() -> Result<UpdateCheckResponse, String> {
    let response = reqwest::Client::new()
        .get(format!("{CLOUD_BRIDGE_URL}/releases/windows"))
        .query(&[("currentVersion", env!("CARGO_PKG_VERSION"))])
        .send()
        .await
        .map_err(|e| format!("Could not check for updates: {e}"))?;
    read_bridge_response(response, "Could not check for updates").await
}

#[tauri::command]
async fn open_update_download(app: tauri::AppHandle, download_url: String) -> Result<String, String> {
    if !download_url.starts_with("https://") {
        return Err("Update download URL is not secure.".to_string());
    }
    app.opener()
        .open_url(download_url, None::<&str>)
        .map_err(|e| format!("Could not open update download: {e}"))?;
    Ok("Opening the official update download in your browser.".to_string())
}

#[tauri::command]
async fn list_installed_printers() -> Result<Vec<String>, String> {
    unsafe {
        let mut needed: u32 = 0; let mut returned: u32 = 0;
        EnumPrintersW(PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS, ptr::null_mut(), 4, ptr::null_mut(), 0, &mut needed, &mut returned);
        if needed == 0 { return Ok(vec![]); }
        let mut buffer = vec![0u8; needed as usize];
        if EnumPrintersW(PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS, ptr::null_mut(), 4, buffer.as_mut_ptr(), needed, &mut needed, &mut returned) == 0 {
            return Err("Could not list installed Windows printers.".to_string());
        }
        let printers = std::slice::from_raw_parts(buffer.as_ptr() as *const PRINTER_INFO_4W, returned as usize);
        Ok(printers.iter().map(|p| ptr_to_string(p.pPrinterName)).filter(|n| !n.is_empty()).collect())
    }
}

#[tauri::command]
async fn pair_device(app: tauri::AppHandle, pairing_code: String, settings: BridgeSettings) -> Result<String, String> {
    if pairing_code.trim().is_empty() { return Err("Pairing code is required.".to_string()); }
    let existing = load_settings_from_disk(&app).unwrap_or_default();
    let request = PairRequest { pairing_code, device_name: settings.device_name.clone(), platform: "windows".to_string(), app_version: env!("CARGO_PKG_VERSION").to_string() };
    let client = reqwest::Client::new();
    let mut call = client.post(format!("{CLOUD_BRIDGE_URL}/pair")).json(&request);
    if !existing.device_token.trim().is_empty() {
        call = call.bearer_auth(existing.device_token);
    }
    let response = call.send().await.map_err(|e| format!("Could not call pairing endpoint: {e}"))?;
    let response: PairResponse = read_bridge_response(response, "Could not pair device").await?;
    let mut saved = settings;
    saved.device_id = response.device_id;
    saved.device_token = response.device_token;
    save_settings_to_disk(&app, &saved)?;
    Ok(format!("Device paired as {}.", saved.device_name))
}

#[tauri::command]
async fn test_receipt_connection(settings: BridgeSettings) -> Result<String, String> {
    match settings.receipt_connection_type {
        ConnectionType::Lan => { let _ = connect_lan_printer(&settings.receipt_printer_target, settings.printer_port).await?; }
        ConnectionType::Windows => validate_windows_printer(&settings.receipt_printer_target)?,
    }
    Ok("Receipt printer route looks valid.".to_string())
}

#[tauri::command]
async fn print_sample_receipt(settings: BridgeSettings) -> Result<String, String> {
    send_to_target(&settings.receipt_connection_type, &settings.receipt_printer_target, settings.printer_port, &sample_receipt_bytes()).await?;
    Ok("Sample receipt sent.".to_string())
}

#[tauri::command]
async fn print_sample_kot(settings: BridgeSettings) -> Result<String, String> {
    if settings.kot_printer_target.trim().is_empty() {
        send_to_target(&settings.receipt_connection_type, &settings.receipt_printer_target, settings.printer_port, &sample_kot_bytes()).await?;
    } else {
        send_to_target(&settings.kot_connection_type, &settings.kot_printer_target, settings.printer_port, &sample_kot_bytes()).await?;
    }
    Ok("Sample KOT sent.".to_string())
}

#[tauri::command]
async fn kick_drawer(settings: BridgeSettings) -> Result<String, String> {
    send_to_target(&settings.receipt_connection_type, &settings.receipt_printer_target, settings.printer_port, DRAWER_KICK).await?;
    Ok("Cash drawer kick sent.".to_string())
}

#[tauri::command]
async fn print_receipt_payload(app: tauri::AppHandle, payload: Value) -> Result<String, String> {
    let settings = load_settings_from_disk(&app)?;
    let bytes = receipt_bytes_from_payload(&payload);
    send_to_target(
        &settings.receipt_connection_type,
        &settings.receipt_printer_target,
        settings.printer_port,
        &bytes,
    )
    .await?;
    Ok("Receipt sent.".to_string())
}

#[tauri::command]
async fn print_kot_payload(app: tauri::AppHandle, payload: Value) -> Result<String, String> {
    let settings = load_settings_from_disk(&app)?;
    let bytes = kot_bytes_from_payload(&payload);
    if settings.kot_printer_target.trim().is_empty() {
        send_to_target(
            &settings.receipt_connection_type,
            &settings.receipt_printer_target,
            settings.printer_port,
            &bytes,
        )
        .await?;
    } else {
        send_to_target(
            &settings.kot_connection_type,
            &settings.kot_printer_target,
            settings.printer_port,
            &bytes,
        )
        .await?;
    }
    Ok("Kitchen ticket sent.".to_string())
}

#[tauri::command]
async fn open_cash_drawer(app: tauri::AppHandle) -> Result<String, String> {
    let settings = load_settings_from_disk(&app)?;
    send_to_target(
        &settings.receipt_connection_type,
        &settings.receipt_printer_target,
        settings.printer_port,
        DRAWER_KICK,
    )
    .await?;
    Ok("Cash drawer kick sent.".to_string())
}

#[tauri::command]
async fn poll_jobs_once(app: tauri::AppHandle) -> Result<String, String> {
    let settings = load_settings_from_disk(&app)?;
    let token = settings.device_token.trim();
    if token.is_empty() { return Err("Device is not paired. Enter the pairing code first.".to_string()); }
    let response = reqwest::Client::new().post(format!("{CLOUD_BRIDGE_URL}/jobs/poll")).bearer_auth(token).send().await
        .map_err(|e| format!("Could not poll jobs: {e}"))?;
    let response: PollResponse = read_bridge_response(response, "Could not poll jobs").await?;
    let count = response.jobs.len();
    let mut failed = 0;
    for job in response.jobs {
        match execute_job(&settings, &job).await {
            Ok(_) => {
                post_job_status(&settings, &job.id, "complete", None).await?;
            }
            Err(print_error) => {
                post_job_status(&settings, &job.id, "fail", Some(print_error.clone())).await
                    .map_err(|status_error| format!("Print failed: {print_error}. Could not report failure: {status_error}"))?;
                failed += 1;
            }
        }
    }
    if failed > 0 {
        return Err(format!("Processed {count} cloud job(s); {failed} print job(s) failed. Check printer routing."));
    }
    Ok(format!("Polled cloud and processed {count} job(s)."))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let settings = load_settings_from_disk(app.handle()).unwrap_or_default();
            apply_launch_on_startup(app.handle(), settings.launch_on_startup)
                .map_err(std::io::Error::other)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            check_for_updates,
            open_update_download,
            list_installed_printers,
            pair_device,
            test_receipt_connection,
            print_sample_receipt,
            print_sample_kot,
            kick_drawer,
            print_receipt_payload,
            print_kot_payload,
            open_cash_drawer,
            poll_jobs_once
        ])
        .run(tauri::generate_context!())
        .expect("error while running Biz Suite Cloud POS");
}
