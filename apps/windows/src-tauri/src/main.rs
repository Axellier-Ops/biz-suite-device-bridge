use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::net::SocketAddr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;
use std::time::Duration;
use tauri::Manager;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum ConnectionType {
    Lan,
    Windows,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BridgeSettings {
    cloud_base_url: String,
    device_token: String,
    device_id: String,
    receipt_connection_type: ConnectionType,
    receipt_printer_target: String,
    kot_connection_type: ConnectionType,
    kot_printer_target: String,
    printer_port: u16,
    device_name: String,
}

impl Default for BridgeSettings {
    fn default() -> Self {
        Self {
            cloud_base_url: String::new(),
            device_token: String::new(),
            device_id: String::new(),
            receipt_connection_type: ConnectionType::Lan,
            receipt_printer_target: String::new(),
            kot_connection_type: ConnectionType::Lan,
            kot_printer_target: String::new(),
            printer_port: 9100,
            device_name: "Front Counter PC".to_string(),
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
    serde_json::from_str(&content).map_err(|e| format!("Settings file is invalid: {e}"))
}

fn save_settings_to_disk(app: &tauri::AppHandle, settings: &BridgeSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let content = serde_json::to_string_pretty(settings).map_err(|e| format!("Could not encode settings: {e}"))?;
    fs::write(path, content).map_err(|e| format!("Could not save settings: {e}"))
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
    let mut handle: HANDLE = 0;
    let printer_name_w = wide(printer_name);
    let doc_name_w = wide("Biz-Suite Device Bridge Job");
    let raw_w = wide("RAW");

    unsafe {
        if OpenPrinterW(printer_name_w.as_ptr(), &mut handle, ptr::null_mut()) == 0 {
            return Err(format!("Could not open Windows printer '{printer_name}'. Check it is installed and online."));
        }
        let mut doc = DOC_INFO_1W { pDocName: doc_name_w.as_ptr() as *mut u16, pOutputFile: ptr::null_mut(), pDatatype: raw_w.as_ptr() as *mut u16 };
        if StartDocPrinterW(handle, 1, &mut doc as *mut _ as *mut u8) == 0 {
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

async fn send_to_target(connection_type: &ConnectionType, target: &str, port: u16, bytes: &[u8]) -> Result<(), String> {
    match connection_type {
        ConnectionType::Lan => send_lan(target, port, bytes).await,
        ConnectionType::Windows => send_windows_printer(target, bytes),
    }
}

fn line(label: &str, value: &str) -> Vec<u8> { format!("{:<18}{:>14}\n", label, value).into_bytes() }

fn sample_receipt_bytes() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(ESC_INIT); b.extend_from_slice(ESC_ALIGN_CENTER); b.extend_from_slice(ESC_BOLD_ON);
    b.extend_from_slice(b"BIZ-SUITE CLOUD\n"); b.extend_from_slice(ESC_BOLD_OFF);
    b.extend_from_slice(b"Sample Receipt\nDevice Bridge Test\n\n"); b.extend_from_slice(ESC_ALIGN_LEFT);
    b.extend_from_slice(b"Order: TEST-1001\nCashier: Test User\n--------------------------------\n");
    b.extend_from_slice(b"2 x Chicken Kottu        24.00\n1 x Lime Juice            5.50\n--------------------------------\n");
    b.extend(line("Subtotal", "29.50").as_slice()); b.extend(line("Service", "2.95").as_slice()); b.extend(line("Total", "32.45").as_slice());
    b.extend_from_slice(b"\nPayment: CASH\n\nThank you.\n\n\n"); b.extend_from_slice(ESC_CUT); b
}

fn sample_kot_bytes() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(ESC_INIT); b.extend_from_slice(ESC_ALIGN_CENTER); b.extend_from_slice(ESC_BOLD_ON);
    b.extend_from_slice(b"KITCHEN ORDER TICKET\n"); b.extend_from_slice(ESC_BOLD_OFF);
    b.extend_from_slice(b"Table 04 | TEST-1001\n\n"); b.extend_from_slice(ESC_ALIGN_LEFT);
    b.extend_from_slice(b"--------------------------------\n"); b.extend_from_slice(ESC_BOLD_ON); b.extend_from_slice(b"2 x Chicken Kottu\n"); b.extend_from_slice(ESC_BOLD_OFF);
    b.extend_from_slice(b"    Note: no chilli\n"); b.extend_from_slice(ESC_BOLD_ON); b.extend_from_slice(b"1 x Lime Juice\n"); b.extend_from_slice(ESC_BOLD_OFF);
    b.extend_from_slice(b"--------------------------------\nPrinted by Biz-Suite Device Bridge\n\n\n"); b.extend_from_slice(ESC_CUT); b
}

fn bytes_from_job(job: &DeviceJob) -> Vec<u8> {
    match job.job_type.as_str() {
        "kot_print" => sample_kot_bytes(),
        "drawer_kick" => DRAWER_KICK.to_vec(),
        _ => sample_receipt_bytes(),
    }
}

async fn execute_job(settings: &BridgeSettings, job: &DeviceJob) -> Result<(), String> {
    let bytes = bytes_from_job(job);
    if job.job_type == "drawer_kick" || job.printer_role.as_deref() == Some("receipt") || job.job_type == "receipt_print" {
        send_to_target(&settings.receipt_connection_type, &settings.receipt_printer_target, settings.printer_port, &bytes).await
    } else {
        let target = if settings.kot_printer_target.trim().is_empty() { &settings.receipt_printer_target } else { &settings.kot_printer_target };
        send_to_target(&settings.kot_connection_type, target, settings.printer_port, &bytes).await
    }
}

async fn post_job_status(settings: &BridgeSettings, job_id: &str, status: &str, error: Option<String>) -> Result<(), String> {
    let base = settings.cloud_base_url.trim().trim_end_matches('/');
    let token = settings.device_token.trim();
    if base.is_empty() || token.is_empty() { return Ok(()); }
    let url = format!("{base}/jobs/{job_id}/{status}");
    let body = serde_json::json!({ "error": error });
    reqwest::Client::new().post(url).bearer_auth(token).json(&body).send().await
        .map_err(|e| format!("Could not report job status: {e}"))?;
    Ok(())
}

#[tauri::command]
async fn load_settings(app: tauri::AppHandle) -> Result<BridgeSettings, String> { load_settings_from_disk(&app) }

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, mut settings: BridgeSettings) -> Result<String, String> {
    let existing = load_settings_from_disk(&app).unwrap_or_default();
    settings.device_token = existing.device_token;
    settings.device_id = existing.device_id;
    if settings.printer_port == 0 { return Err("Printer port is invalid.".to_string()); }
    save_settings_to_disk(&app, &settings)?;
    Ok("Settings saved.".to_string())
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
    let base = settings.cloud_base_url.trim().trim_end_matches('/');
    if base.is_empty() { return Err("Cloud bridge URL is required.".to_string()); }
    let request = PairRequest { pairing_code, device_name: settings.device_name.clone(), platform: "windows".to_string(), app_version: env!("CARGO_PKG_VERSION").to_string() };
    let response: PairResponse = reqwest::Client::new().post(format!("{base}/pair")).json(&request).send().await
        .map_err(|e| format!("Could not call pairing endpoint: {e}"))?
        .json().await.map_err(|e| format!("Pairing response was invalid: {e}"))?;
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
        ConnectionType::Windows => { if settings.receipt_printer_target.trim().is_empty() { return Err("Windows printer name is required.".to_string()); } }
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
    let target = if settings.kot_printer_target.trim().is_empty() { &settings.receipt_printer_target } else { &settings.kot_printer_target };
    send_to_target(&settings.kot_connection_type, target, settings.printer_port, &sample_kot_bytes()).await?;
    Ok("Sample KOT sent.".to_string())
}

#[tauri::command]
async fn kick_drawer(settings: BridgeSettings) -> Result<String, String> {
    send_to_target(&settings.receipt_connection_type, &settings.receipt_printer_target, settings.printer_port, DRAWER_KICK).await?;
    Ok("Cash drawer kick sent.".to_string())
}

#[tauri::command]
async fn poll_jobs_once(app: tauri::AppHandle) -> Result<String, String> {
    let settings = load_settings_from_disk(&app)?;
    let base = settings.cloud_base_url.trim().trim_end_matches('/');
    let token = settings.device_token.trim();
    if base.is_empty() || token.is_empty() { return Err("Device is not paired. Add cloud URL and pair first.".to_string()); }
    let response: PollResponse = reqwest::Client::new().post(format!("{base}/jobs/poll")).bearer_auth(token).send().await
        .map_err(|e| format!("Could not poll jobs: {e}"))?
        .json().await.map_err(|e| format!("Poll response was invalid: {e}"))?;
    let count = response.jobs.len();
    for job in response.jobs {
        match execute_job(&settings, &job).await {
            Ok(_) => { let _ = post_job_status(&settings, &job.id, "complete", None).await; }
            Err(e) => { let _ = post_job_status(&settings, &job.id, "fail", Some(e)).await; }
        }
    }
    Ok(format!("Polled cloud and processed {count} job(s)."))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![load_settings, save_settings, list_installed_printers, pair_device, test_receipt_connection, print_sample_receipt, print_sample_kot, kick_drawer, poll_jobs_once])
        .run(tauri::generate_context!())
        .expect("error while running Biz-Suite Device Bridge");
}
