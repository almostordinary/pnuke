#!/usr/bin/env node

const { spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const binName = process.platform === "win32" ? "pnuke.exe" : "pnuke";
const binPath = path.join(__dirname, binName);

async function run() {
  if (!fs.existsSync(binPath)) {
    console.log("⚡ Fetching native pnuke binary...");
    try {
      const install = require("../scripts/install.js");
      await install();
    } catch (err) {
      console.error("Failed to download native pnuke binary:", err.message);
      process.exit(1);
    }
  }

  const result = spawnSync(binPath, process.argv.slice(2), {
    stdio: "inherit",
  });

  process.exit(result.status ?? 0);
}

run();
