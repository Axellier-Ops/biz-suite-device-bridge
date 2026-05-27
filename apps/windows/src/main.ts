import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

type ConnectionType = "lan" | "windows";

type BridgeSettings = {
  deviceToken: string;
  deviceId: string;
  receiptConnectionType: ConnectionType;
  receiptPrinterTarget: string;
  kotConnectionType: ConnectionType;
  kotPrinterTarget: string;
  printerPort: number;
  deviceName: string;
};

const statusPill = document.getElementById("status-pill")!;
const logEl = document.getElementById("log")!;
const receiptConnectionTypeEl = document.getElementById("receipt-connection-type") as HTMLSelectElement;
const kotConnectionTypeEl = document.getElementById("kot-connection-type") as HTMLSelectElement;
const receiptPrinterTargetEl = document.getElementById("receipt-printer-target") as HTMLInputElement;
const kotPrinterTargetEl = document.getElementById("kot-printer-target") as HTMLInputElement;
const printerPortEl = document.getElementById("printer-port") as HTMLInputElement;
const pairingCodeEl = document.getElementById("pairing-code") as HTMLInputElement;
const deviceNameEl = document.getElementById("device-name") as HTMLInputElement;
const installedPrintersEl = document.getElementById("installed-printers") as HTMLSelectElement;
const pairingStatusEl = document.getElementById("pairing-status")!;
const pairingHelpEl = document.getElementById("pairing-help")!;
const pairButtonEl = document.getElementById("pair") as HTMLButtonElement;
const repairButtonEl = document.getElementById("repair") as HTMLButtonElement;
const pollOnceButtonEl = document.getElementById("poll-once") as HTMLButtonElement;
const pollLoopButtonEl = document.getElementById("poll-loop") as HTMLButtonElement;

let polling = false;

const thermalPrinterNameHints = [
  "receipt",
  "thermal",
  "pos",
  "epson",
  "tm-",
  "bixolon",
  "star",
  "citizen",
  "xprinter",
  "rongta",
  "gprinter",
  "munbyn",
];

function setStatus(text: string, mode: "idle" | "ok" | "error" = "idle") {
  statusPill.textContent = text;
  statusPill.className = `pill ${mode}`;
}

function log(message: string) {
  const timestamp = new Date().toLocaleTimeString();
  logEl.textContent = `[${timestamp}] ${message}\n${logEl.textContent}`;
}

function setPairingState(isPaired: boolean, editing = false) {
  const identityLocked = isPaired && !editing;
  pairingCodeEl.value = "";
  pairingCodeEl.disabled = identityLocked;
  pairingCodeEl.placeholder = identityLocked ? "Already paired" : "842913";
  deviceNameEl.disabled = identityLocked;
  pairButtonEl.classList.toggle("hidden", identityLocked);
  pairButtonEl.textContent = isPaired ? "Apply new pairing" : "Pair with cloud";
  repairButtonEl.classList.toggle("hidden", !identityLocked);
  pollOnceButtonEl.disabled = !isPaired;
  pollLoopButtonEl.disabled = !isPaired;
  pairingStatusEl.textContent = isPaired
    ? editing
      ? "Paired - changing"
      : "Paired"
    : "Not paired";
  pairingStatusEl.className = `pair-state ${isPaired ? "paired" : "not-paired"}`;
  pairingHelpEl.textContent = identityLocked
    ? "This bridge is securely paired. Pair again only when moving it to another register or business."
    : "Connects securely to Biz-Suite Cloud. Enter the one-time pairing code from your POS settings.";
}

function getPort(): number {
  const value = Number(printerPortEl.value || "9100");
  return Number.isFinite(value) && value > 0 ? value : 9100;
}

function getSettings(): BridgeSettings {
  return {
    deviceToken: "",
    deviceId: "",
    receiptConnectionType: receiptConnectionTypeEl.value as ConnectionType,
    receiptPrinterTarget: receiptPrinterTargetEl.value.trim(),
    kotConnectionType: kotConnectionTypeEl.value as ConnectionType,
    kotPrinterTarget: kotPrinterTargetEl.value.trim(),
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
  receiptConnectionTypeEl.value = settings.receiptConnectionType || "lan";
  kotConnectionTypeEl.value = settings.kotConnectionType || "lan";
  receiptPrinterTargetEl.value = settings.receiptPrinterTarget || "";
  kotPrinterTargetEl.value = settings.kotPrinterTarget || "";
  printerPortEl.value = String(settings.printerPort || 9100);
  deviceNameEl.value = settings.deviceName || "Front Counter PC";
  setPairingState(Boolean(settings.deviceToken));
  log(settings.deviceToken ? "Settings loaded. Device is paired." : "Settings loaded. Device is not paired.");
}

function autoDetectedPrinter(printers: string[]): string | null {
  const likelyThermalPrinters = printers.filter((printer) => {
    const normalized = printer.toLowerCase();
    return thermalPrinterNameHints.some((hint) => normalized.includes(hint));
  });

  if (likelyThermalPrinters.length === 1) return likelyThermalPrinters[0];
  if (likelyThermalPrinters.length === 0 && printers.length === 1) return printers[0];
  return null;
}

async function refreshPrinters(autoConfigure = false) {
  const printers = await invoke<string[]>("list_installed_printers");
  installedPrintersEl.innerHTML = "";
  for (const printer of printers) {
    const option = document.createElement("option");
    option.value = printer;
    option.textContent = printer;
    installedPrintersEl.appendChild(option);
  }
  log(`Loaded ${printers.length} installed printer(s).`);

  if (!autoConfigure || receiptPrinterTargetEl.value.trim()) return;

  const detectedPrinter = autoDetectedPrinter(printers);
  if (!detectedPrinter) {
    if (printers.length > 0) {
      log("Multiple printers found. Select the receipt printer to prevent incorrect routing.");
    }
    return;
  }

  receiptConnectionTypeEl.value = "windows";
  receiptPrinterTargetEl.value = detectedPrinter;
  if (!kotPrinterTargetEl.value.trim()) {
    kotConnectionTypeEl.value = "windows";
    kotPrinterTargetEl.value = detectedPrinter;
  }
  await invoke<string>("save_settings", { settings: getSettings() });
  log(`Auto-configured Windows printer: ${detectedPrinter}.`);
}

document.getElementById("save-settings")?.addEventListener("click", () => {
  runAction("Save printer settings", async () => invoke<string>("save_settings", { settings: getSettings() }));
});

document.getElementById("refresh-printers")?.addEventListener("click", () => {
  runAction("Refresh installed printers", async () => {
    await refreshPrinters(true);
    return "Installed printers refreshed.";
  });
});

document.getElementById("use-selected-receipt")?.addEventListener("click", () => {
  if (!installedPrintersEl.value) {
    log("No installed printer is selected. Refresh printers and select one first.");
    return;
  }
  receiptConnectionTypeEl.value = "windows";
  receiptPrinterTargetEl.value = installedPrintersEl.value;
  log(`Receipt printer set to ${installedPrintersEl.value}.`);
});

document.getElementById("use-selected-kot")?.addEventListener("click", () => {
  if (!installedPrintersEl.value) {
    log("No installed printer is selected. Refresh printers and select one first.");
    return;
  }
  kotConnectionTypeEl.value = "windows";
  kotPrinterTargetEl.value = installedPrintersEl.value;
  log(`KOT printer set to ${installedPrintersEl.value}.`);
});

document.getElementById("test-connection")?.addEventListener("click", () => {
  runAction("Connection test", async () => invoke<string>("test_receipt_connection", { settings: getSettings() }));
});

document.getElementById("sample-receipt")?.addEventListener("click", () => {
  runAction("Sample receipt", async () => invoke<string>("print_sample_receipt", { settings: getSettings() }));
});

document.getElementById("sample-kot")?.addEventListener("click", () => {
  runAction("Sample KOT", async () => invoke<string>("print_sample_kot", { settings: getSettings() }));
});

document.getElementById("drawer-kick")?.addEventListener("click", () => {
  runAction("Cash drawer kick", async () => invoke<string>("kick_drawer", { settings: getSettings() }));
});

document.getElementById("pair")?.addEventListener("click", () => {
  runAction("Cloud pairing", async () => {
    const result = await invoke<string>("pair_device", {
      pairingCode: pairingCodeEl.value.trim(),
      settings: getSettings(),
    });
    setPairingState(true);
    return result;
  });
});

document.getElementById("repair")?.addEventListener("click", () => {
  setPairingState(true, true);
  log("Existing pairing remains active until a new one-time pairing code succeeds.");
});

document.getElementById("poll-once")?.addEventListener("click", () => {
  runAction("Poll once", async () => invoke<string>("poll_jobs_once"));
});

document.getElementById("poll-loop")?.addEventListener("click", async () => {
  if (polling) {
    polling = false;
    pollLoopButtonEl.textContent = "Run polling loop";
    setStatus("Paired", "ok");
    log("Polling loop stopped.");
    return;
  }
  polling = true;
  pollLoopButtonEl.textContent = "Stop polling loop";
  setStatus("Polling", "ok");
  log("Polling loop started. Keep this app open.");
  while (polling) {
    try {
      const result = await invoke<string>("poll_jobs_once");
      log(result);
    } catch (error) {
      log(`Polling error: ${String(error)}`);
    }
    await new Promise((resolve) => setTimeout(resolve, 5000));
  }
});

loadSettings()
  .then(() => refreshPrinters(true))
  .catch((error) => {
    setStatus("Error", "error");
    log(`Startup error: ${String(error)}`);
  });
