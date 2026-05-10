import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

type BridgeSettings = {
  receiptPrinterIp: string;
  kotPrinterIp: string;
  printerPort: number;
  deviceName: string;
};

const statusPill = document.getElementById("status-pill")!;
const logEl = document.getElementById("log")!;
const receiptPrinterIpEl = document.getElementById("receipt-printer-ip") as HTMLInputElement;
const kotPrinterIpEl = document.getElementById("kot-printer-ip") as HTMLInputElement;
const printerPortEl = document.getElementById("printer-port") as HTMLInputElement;
const pairingCodeEl = document.getElementById("pairing-code") as HTMLInputElement;
const deviceNameEl = document.getElementById("device-name") as HTMLInputElement;

function setStatus(text: string, mode: "idle" | "ok" | "error" = "idle") {
  statusPill.textContent = text;
  statusPill.className = `pill ${mode}`;
}

function log(message: string) {
  const timestamp = new Date().toLocaleTimeString();
  logEl.textContent = `[${timestamp}] ${message}\n${logEl.textContent}`;
}

function getPort(): number {
  const value = Number(printerPortEl.value || "9100");
  return Number.isFinite(value) && value > 0 ? value : 9100;
}

function getSettings(): BridgeSettings {
  return {
    receiptPrinterIp: receiptPrinterIpEl.value.trim(),
    kotPrinterIp: kotPrinterIpEl.value.trim(),
    printerPort: getPort(),
    deviceName: deviceNameEl.value.trim() || "Front Counter PC",
  };
}

async function runAction(label: string, action: () => Promise<string>) {
  try {
    setStatus("Working…", "idle");
    log(`${label} started.`);
    const result = await action();
    setStatus("OK", "ok");
    log(result);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    setStatus("Error", "error");
    log(message);
  }
}

async function loadSettings() {
  const settings = await invoke<BridgeSettings>("load_settings");
  receiptPrinterIpEl.value = settings.receiptPrinterIp || "";
  kotPrinterIpEl.value = settings.kotPrinterIp || "";
  printerPortEl.value = String(settings.printerPort || 9100);
  deviceNameEl.value = settings.deviceName || "Front Counter PC";
  log("Settings loaded.");
}

document.getElementById("save-settings")?.addEventListener("click", () => {
  runAction("Save settings", async () => {
    return invoke<string>("save_settings", { settings: getSettings() });
  });
});

document.getElementById("test-connection")?.addEventListener("click", () => {
  runAction("Connection test", async () => {
    const settings = getSettings();
    return invoke<string>("test_connection", {
      printerIp: settings.receiptPrinterIp,
      port: settings.printerPort,
    });
  });
});

document.getElementById("sample-receipt")?.addEventListener("click", () => {
  runAction("Sample receipt", async () => {
    const settings = getSettings();
    return invoke<string>("print_sample_receipt", {
      printerIp: settings.receiptPrinterIp,
      port: settings.printerPort,
    });
  });
});

document.getElementById("sample-kot")?.addEventListener("click", () => {
  runAction("Sample KOT", async () => {
    const settings = getSettings();
    return invoke<string>("print_sample_kot", {
      printerIp: settings.kotPrinterIp || settings.receiptPrinterIp,
      port: settings.printerPort,
    });
  });
});

document.getElementById("drawer-kick")?.addEventListener("click", () => {
  runAction("Cash drawer kick", async () => {
    const settings = getSettings();
    return invoke<string>("kick_drawer", {
      printerIp: settings.receiptPrinterIp,
      port: settings.printerPort,
    });
  });
});

document.getElementById("pair")?.addEventListener("click", () => {
  runAction("Pair placeholder", async () => {
    const code = pairingCodeEl.value.trim();
    const settings = getSettings();
    return invoke<string>("pair_device", {
      pairingCode: code,
      deviceName: settings.deviceName,
    });
  });
});

loadSettings().catch((error) => {
  setStatus("Error", "error");
  log(`Could not load settings: ${String(error)}`);
});
