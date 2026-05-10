use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

const ESC_INIT: &[u8] = b"\x1B\x40";
const ESC_CUT: &[u8] = b"\x1D\x56\x00";
const DRAWER_KICK: &[u8] = b"\x1B\x70\x00\x19\xFA";

#[tauri::command]
async fn pair_device(pairing_code: String) -> Result<String, String> {
    if pairing_code.trim().is_empty() {
        return Err("Pairing code is required.".to_string());
    }

    // TODO:
    // 1. Call Biz-Suite Cloud pairing endpoint
    // 2. Exchange pairing code for device token
    // 3. Store token in OS secure storage
    // 4. Start heartbeat/job polling
    Ok(format!("Paired with code {}", pairing_code))
}

#[tauri::command]
async fn test_print(printer_ip: String) -> Result<String, String> {
    let mut stream = TcpStream::connect(format!("{}:9100", printer_ip))
        .await
        .map_err(|e| format!("Could not connect to printer: {}", e))?;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(ESC_INIT);
    bytes.extend_from_slice(b"Biz-Suite Device Bridge\n");
    bytes.extend_from_slice(b"Test Print\n");
    bytes.extend_from_slice(b"--------------------------\n");
    bytes.extend_from_slice(b"Printer connected successfully.\n\n\n");
    bytes.extend_from_slice(ESC_CUT);

    stream
        .write_all(&bytes)
        .await
        .map_err(|e| format!("Could not print: {}", e))?;

    Ok("Test print sent.".to_string())
}

#[tauri::command]
async fn kick_drawer(printer_ip: String) -> Result<String, String> {
    let mut stream = TcpStream::connect(format!("{}:9100", printer_ip))
        .await
        .map_err(|e| format!("Could not connect to printer: {}", e))?;

    stream
        .write_all(DRAWER_KICK)
        .await
        .map_err(|e| format!("Could not open cash drawer: {}", e))?;

    Ok("Cash drawer kick sent.".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![pair_device, test_print, kick_drawer])
        .run(tauri::generate_context!())
        .expect("error while running Biz-Suite Device Bridge");
}
