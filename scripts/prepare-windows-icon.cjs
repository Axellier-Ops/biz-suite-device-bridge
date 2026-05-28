const fs = require("node:fs");
const path = require("node:path");

const appDir = process.argv[2];
if (!appDir) {
  console.error("Usage: node scripts/prepare-windows-icon.cjs <app-dir>");
  process.exit(1);
}

const iconDir = path.join(appDir, "src-tauri", "icons");
const iconPath = path.join(iconDir, "icon.ico");

fs.mkdirSync(iconDir, { recursive: true });

const size = 16;
const xorSize = size * size * 4;
const andStride = 4;
const andSize = andStride * size;
const imageSize = 40 + xorSize + andSize;
const buffer = Buffer.alloc(6 + 16 + imageSize);

let offset = 0;
buffer.writeUInt16LE(0, offset);
offset += 2;
buffer.writeUInt16LE(1, offset);
offset += 2;
buffer.writeUInt16LE(1, offset);
offset += 2;

buffer.writeUInt8(size, offset++);
buffer.writeUInt8(size, offset++);
buffer.writeUInt8(0, offset++);
buffer.writeUInt8(0, offset++);
buffer.writeUInt16LE(1, offset);
offset += 2;
buffer.writeUInt16LE(32, offset);
offset += 2;
buffer.writeUInt32LE(imageSize, offset);
offset += 4;
buffer.writeUInt32LE(22, offset);
offset += 4;

buffer.writeUInt32LE(40, offset);
offset += 4;
buffer.writeInt32LE(size, offset);
offset += 4;
buffer.writeInt32LE(size * 2, offset);
offset += 4;
buffer.writeUInt16LE(1, offset);
offset += 2;
buffer.writeUInt16LE(32, offset);
offset += 2;
buffer.writeUInt32LE(0, offset);
offset += 4;
buffer.writeUInt32LE(xorSize + andSize, offset);
offset += 4;
buffer.writeInt32LE(0, offset);
offset += 4;
buffer.writeInt32LE(0, offset);
offset += 4;
buffer.writeUInt32LE(0, offset);
offset += 4;
buffer.writeUInt32LE(0, offset);
offset += 4;

for (let y = 0; y < size; y += 1) {
  for (let x = 0; x < size; x += 1) {
    const border = x < 2 || y < 2 || x >= size - 2 || y >= size - 2;
    const accent = x >= 5 && x <= 10 && y >= 5 && y <= 10;
    const color = accent
      ? { r: 31, g: 122, b: 92 }
      : border
        ? { r: 23, g: 32, b: 38 }
        : { r: 255, g: 255, b: 255 };

    buffer.writeUInt8(color.b, offset++);
    buffer.writeUInt8(color.g, offset++);
    buffer.writeUInt8(color.r, offset++);
    buffer.writeUInt8(255, offset++);
  }
}

fs.writeFileSync(iconPath, buffer);
console.log(`Prepared Windows icon: ${iconPath}`);
