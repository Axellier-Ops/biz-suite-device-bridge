import { invoke } from "@tauri-apps/api/core";

const statusEl = document.getElementById("status")!;
const pairingCodeEl = document.getElementById("pairing-code") as HTMLInputElement;
const printerIpEl = document.getElementById("printer-ip") as HTMLInputElement;

document.getElementById("pair")?.addEventListener("click", async () => {
  const code = pairingCodeEl.value.trim();
  const result = await invoke<string>("pair_device", { pairingCode: code });
  statusEl.textContent = result;
});

document.getElementById("test-print")?.addEventListener("click", async () => {
  const ip = printerIpEl.value.trim();
  const result = await invoke<string>("test_print", { printerIp: ip });
  statusEl.textContent = result;
});

document.getElementById("drawer-kick")?.addEventListener("click", async () => {
  const ip = printerIpEl.value.trim();
  const result = await invoke<string>("kick_drawer", { printerIp: ip });
  statusEl.textContent = result;
});
