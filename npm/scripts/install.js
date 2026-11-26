const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const https = require("https");

const pkg = require("../package.json");
const VERSION = pkg.version;
const REPO = "almostordinary/pnuke";

function getTarget() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "linux") {
    if (arch === "x64") return "x86_64-unknown-linux-gnu";
    if (arch === "arm64") return "aarch64-unknown-linux-gnu";
  } else if (platform === "darwin") {
    if (arch === "arm64") return "aarch64-apple-darwin";
    if (arch === "x64") return "x86_64-apple-darwin";
  } else if (platform === "win32") {
    if (arch === "x64") return "x86_64-pc-windows-msvc";
  }

  throw new Error(`Unsupported platform: ${platform} (${arch})`);
}

async function download(url, dest) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          return download(res.headers.location, dest).then(resolve).catch(reject);
        }
        if (res.statusCode !== 200) {
          return reject(new Error(`Failed to download binary: HTTP ${res.statusCode}`));
        }
        const fileStream = fs.createWriteStream(dest);
        res.pipe(fileStream);
        fileStream.on("finish", () => {
          fileStream.close(resolve);
        });
      })
      .on("error", reject);
  });
}

async function main() {
  const target = getTarget();
  const isWindows = process.platform === "win32";
  const archiveExt = isWindows ? "zip" : "tar.gz";
  const binName = isWindows ? "pnuke.exe" : "pnuke";
  const binDir = path.join(__dirname, "..", "bin");
  const binPath = path.join(binDir, binName);

  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  const archiveName = `pnuke-v${VERSION}-${target}.${archiveExt}`;
  const downloadUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${archiveName}`;
  const tempArchive = path.join(binDir, archiveName);

  console.log(`⚡ Downloading pnuke native binary for ${target}...`);
  try {
    await download(downloadUrl, tempArchive);
    if (isWindows) {
      execSync(`tar -xf "${tempArchive}" -C "${binDir}"`);
    } else {
      execSync(`tar -xzf "${tempArchive}" -C "${binDir}"`);
      fs.chmodSync(binPath, 0o755);
    }
    fs.unlinkSync(tempArchive);
    console.log("✔ pnuke native binary installed successfully.");
  } catch (err) {
    console.warn(`Warning: Could not fetch pre-built binary (${err.message}).`);
    console.warn("You can install via Cargo: 'cargo install pnuke'");
  }
}

if (require.main === module) {
  main().catch((err) => {
    console.error(err.message);
    process.exit(1);
  });
}

module.exports = main;
