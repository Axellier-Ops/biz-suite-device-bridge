use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tauri::Manager;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::timeout;

const ESC_INIT: &[u8] = b"\x1B\x40";
const ESC_ALIGN_CENTER: &[u8] = b"\x1B\x61\x01";
const ESC_ALIGN_LEFT: &[u8] = b"\x1B\x61\x00";
const ESC_BOLD_ON: &[u8] = b"\x1B\x45\x01";
const ESC_BOLD_OFF: &[u8] = b"\x1B\x45\x00";
const ESC_CUT: &[u8] = b"\x1D\x56\x00";
const DRAWER_KICK: &[u8] = b"\x1B\x70\x00\x19\xFA";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BridgeSettings {
    receipt_printer_ip: String,
    kot_printer_ip: String,
    printer_port: u16,
    device_name: String,
}

impl Default for BridgeSettings {
    fn default() -> Self {
        Self {
            receipt_printer_ip: String::new(),
            kot_printer_ip: String::new(),
            printer_port: 9100,
            device_name: "Front Counter PC".to_string(),
        }
    }
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Could not locate app config directory: {e}"))?;

    fs::create_dir_all(&dir).map_err(|e| format!("Could not create app config directory: {e}"))?;
    Ok(dir.join("settings.json"))
}

fn validate_printer_target(printer_ip: &str, port: u16) -> Result<(), String> {
    if printer_ip.trim().is_empty() {
        return Err("Printer IP is required.".to_string());
    }

    if port == 0 {
        return Err("Printer port is invalid.".to_string());
    }

    Ok(())
}

async fn connect_printer(printer_ip: &str, port: u16) -> Result<TcpStream, String> {
    validate_printer_target(printer_ip, port)?;

    let address: SocketAddr = format!("{}:{}", printer_ip.trim(), port)
        .parse()
        .map_err(|_| "Printer address is invalid. Use an IP address like 192.168.1.50.".to_string())?;

    timeout(Duration::from_secs(4), TcpStream::connect(address))
        .await
        .map_err(|_| "Printer connection timed out. Check printer IP, power and network.".to_string())?
        .map_err(|e| format!("Could not connect to printer: {e}"))
}

async fn send_to_printer(printer_ip: &str, port: u16, bytes: &[u8]) -> Result<(), String> {
    let mut stream = connect_printer(printer_ip, port).await?;
    stream
        .write_all(bytes)
        .await
        .map_err(|e| format!("Could not write to printer: {e}"))?;
    stream
        .shutdown()
        .await
        .map_err(|e| format!("Could not close printer connection cleanly: {e}"))?;
    Ok(())
}

fn line(label: &str, value: &str) -> Vec<u8> {
    format!("{:<18}{:>14}\n", label, value).into_bytes()
}

fn sample_receipt_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(ESC_INIT);
    bytes.extend_from_slice(ESC_ALIGN_CENTER);
    bytes.extend_from_slice(ESC_BOLD_ON);
    bytes.extend_from_slice(b"BIZ-SUITE CLOUD\n");
    bytes.extend_from_slice(ESC_BOLD_OFF);
    bytes.extend_from_slice(b"Sample Receipt\n");
    bytes.extend_from_slice(b"Device Bridge Test\n\n");
    bytes.extend_from_slice(ESC_ALIGN_LEFT);
    bytes.extend_from_slice(b"Order: TEST-1001\n");
    bytes.extend_from_slice(b"Cashier: Test User\n");
    bytes.extend_from_slice(b"--------------------------------\n");
    bytes.extend_from_slice(b"2 x Chicken Kottu        24.00\n");
    bytes.extend_from_slice(b"1 x Lime Juice            5.50\n");
    bytes.extend_from_slice(b"--------------------------------\n");
    bytes.extend(line("Subtotal", "29.50").as_slice());
    bytes.extend(line("Service", "2.95").as_slice());
    bytes.extend(line("Total", "32.45").as_slice());
    bytes.extend_from_slice(b"\nPayment: CASH\n");
    bytes.extend_from_slice(b"\nThank you.\n\n\n");
    bytes.extend_from_slice(ESC_CUT);
    bytes
}

fn sample_kot_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(ESC_INIT);
    bytes.extend_from_slice(ESC_ALIGN_CENTER);
    bytes.extend_from_slice(ESC_BOLD_ON);
    bytes.extend_from_slice(b"KITCHEN ORDER TICKET\n");
    bytes.extend_from_slice(ESC_BOLD_OFF);
    bytes.extend_from_slice(b"Table 04 | TEST-1001\n\n");
    bytes.extend_from_slice(ESC_ALIGN_LEFT);
    bytes.extend_from_slice(b"--------------------------------\n");
    bytes.extend_from_slice(ESC_BOLD_ON);
    bytes.extend_from_slice(b"2 x Chicken Kottu\n");
    bytes.extend_from_slice(ESC_BOLD_OFF);
    bytes.extend_from_slice(b"    Note: no chilli\n");
    bytes.extend_from_slice(ESC_BOLD_ON);
    bytes.extend_from_slice(b"1 x Lime Juice\n");
    bytes.extend_from_slice(ESC_BOLD_OFF);
    bytes.extend_from_slice(b"--------------------------------\n");
    bytes.extend_from_slice(b"Printed by Biz-Suite Device Bridge\n\n\n");
    bytes.extend_from_slice(ESC_CUT);
    bytes
}

#[tauri::command]
async fn load_settings(app: tauri::AppHandle) -> Result<BridgeSettings, String> {
    let path = settings_path(&app)?;
    if !path.exists() {
        return Ok(BridgeSettings::default());
    }

    let content = fs::read_to_string(path).map_err(|e| format!("Could not read settings: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Settings file is invalid: {e}"))
}

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, settings: BridgeSettings) -> Result<String, String> {
    if settings.printer_port == 0 {
        return Err("Printer port is invalid.".to_string());
    }

    let path = settings_path(&app)?;
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Could not encode settings: {e}"))?;
    fs::write(path, content).map_err(|e| format!("Could not save settings: {e}"))?;
    Ok("Settings saved.".to_string())
}

#[tauri::command]
async fn pair_device(pairing_code: String, device_name: String) -> Result<String, String> {
    if pairing_code.trim().is_empty() {
        return Err("Pairing code is required.".to_string());
    }

    if device_name.trim().is_empty() {
        return Err("Device name is required.".to_string());
    }

    Ok(format!(
        "Pairing placeholder accepted for {device_name}. Cloud pairing endpoint is next."
    ))
}

#[tauri::command]
async fn test_connection(printer_ip: String, port: u16) -> Result<String, String> {
    let _stream = connect_printer(&printer_ip, port).await?;
    Ok(format!("Connected to printer at {}:{}.", printer_ip, port))
}

#[tauri::command]
async fn print_sample_receipt(printer_ip: String, port: u16) -> Result<String, String> {
    send_to_printer(&printer_ip, port, &sample_receipt_bytes()).await?;
    Ok("Sample receipt sent to printer.".to_string())
}

#[tauri::command]
async fn print_sample_kot(printer_ip: String, port: u16) -> Result<String, String> {
    send_to_printer(&printer_ip, port, &sample_kot_bytes()).await?;
    Ok("Sample KOT sent to printer.".to_string())
}

#[tauri::command]
async fn kick_drawer(printer_ip: String, port: u16) -> Result<String, String> {
    send_to_printer(&printer_ip, port, DRAWER_KICK).await?;
    Ok("Cash drawer kick sent.".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            pair_device,
            test_connection,
            print_sample_receipt,
            print_sample_kot,
            kick_drawer
        ])
        .run(tauri::generate_context!())
        .expect("error while running Biz-Suite Device Bridge");
}
