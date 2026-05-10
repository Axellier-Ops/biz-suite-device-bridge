export type DevicePlatform = "windows" | "android";
export type DeviceJobStatus = "pending" | "processing" | "completed" | "failed";
export type DeviceJobType =
  | "receipt_print"
  | "kot_print"
  | "drawer_kick"
  | "test_print";

export type PrinterRole = "receipt" | "kot" | "bar" | "label";
export type PrinterConnectionType = "lan" | "usb" | "bluetooth";

export interface BridgeRegistration {
  deviceId: string;
  tenantId: string;
  locationId: string;
  registerId?: string | null;
  platform: DevicePlatform;
  deviceName: string;
  appVersion: string;
}

export interface PrintDevice {
  id: string;
  name: string;
  role: PrinterRole;
  connectionType: PrinterConnectionType;
  address?: string;
  port?: number;
  isDefault?: boolean;
}

export interface DeviceJob {
  id: string;
  tenantId: string;
  locationId: string;
  moduleKey: string;
  jobType: DeviceJobType;
  printerRole?: PrinterRole;
  payload: unknown;
  status: DeviceJobStatus;
  attempts: number;
  createdAt: string;
}

export interface ReceiptPrintPayload {
  businessName: string;
  address?: string;
  phone?: string;
  orderNumber: string;
  cashierName?: string;
  tableName?: string;
  customerName?: string;
  items: Array<{
    name: string;
    quantity: number;
    unitPrice: number;
    total: number;
    notes?: string | null;
  }>;
  subtotal: number;
  discount: number;
  serviceCharge: number;
  tax: number;
  total: number;
  paymentMethod: "cash" | "card" | "mobile";
  paidAt: string;
}

export interface KotPrintPayload {
  businessName: string;
  orderNumber: string;
  tableName?: string;
  orderType: "dine-in" | "takeaway" | string;
  createdAt: string;
  items: Array<{
    name: string;
    quantity: number;
    notes?: string | null;
    modifiers?: Record<string, string>;
  }>;
}
