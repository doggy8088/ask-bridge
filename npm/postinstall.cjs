#!/usr/bin/env node
'use strict';

const { createHash } = require('node:crypto');
const { spawnSync } = require('node:child_process');
const {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} = require('node:fs');
const { get } = require('node:https');
const { join } = require('node:path');
const { URL } = require('node:url');

const PACKAGE_ROOT = join(__dirname, '..');
const BINARY_NAME = "ask-bridge";
const GITHUB_OWNER = "doggy8088";
const GITHUB_REPO = "ask-bridge";
const BIN_DIR = join(__dirname, `${BINARY_NAME}-bin`);
const BIN_NAME = process.platform === 'win32' ? `${BINARY_NAME}.exe` : BINARY_NAME;
const DEST = join(BIN_DIR, BIN_NAME);

const TARGETS = {
  'darwin-arm64': 'aarch64-apple-darwin',
  'darwin-x64': 'x86_64-apple-darwin',
  'linux-x64': 'x86_64-unknown-linux-gnu',
  'win32-x64': 'x86_64-pc-windows-msvc',
};

function platformKey(platform = process.platform, arch = process.arch) {
  return `${platform}-${arch}`;
}

function cargoTarget(platform = process.platform, arch = process.arch) {
  const target = TARGETS[platformKey(platform, arch)];
  if (!target) {
    throw new Error(`Unsupported platform: ${platform}/${arch}`);
  }
  return target;
}

function packageVersion() {
  return require(join(PACKAGE_ROOT, 'package.json')).version;
}

function artifactName(target) {
  const ext = target.includes('windows') || target.includes('pc-windows') ? 'zip' : 'tar.xz';
  return `${BINARY_NAME}-${target}.${ext}`;
}

function releaseBaseUrl(version = packageVersion()) {
  return `https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/download/v${version}`;
}

function sha256(path) {
  return createHash('sha256').update(readFileSync(path)).digest('hex');
}

function verifyChecksum(filePath, checksumText) {
  const expected = checksumText.trim().split(/\s+/)[0].toLowerCase();
  if (!/^[a-f0-9]{64}$/.test(expected)) {
    throw new Error('Invalid checksum file format');
  }
  const actual = sha256(filePath);
  if (actual !== expected) {
    throw new Error(`Checksum mismatch for ${filePath}: expected ${expected}, got ${actual}`);
  }
}

function download(url, destination, redirectsRemaining = 5) {
  return new Promise((resolve, reject) => {
    get(url, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location && redirectsRemaining > 0) {
        const nextUrl = new URL(res.headers.location, url).toString();
        download(nextUrl, destination, redirectsRemaining - 1).then(resolve, reject);
        return;
      }
      if (res.statusCode !== 200) {
        reject(new Error(`Download failed ${res.statusCode}: ${url}`));
        return;
      }
      const chunks = [];
      res.on('data', (chunk) => chunks.push(chunk));
      res.on('end', () => {
        writeFileSync(destination, Buffer.concat(chunks));
        resolve();
      });
    }).on('error', reject);
  });
}

function run(command, args) {
  const result = spawnSync(command, args, { stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Command failed: ${command}`);
}

function extract(archive, destDir) {
  mkdirSync(destDir, { recursive: true });
  if (archive.endsWith('.zip')) {
    if (process.platform === 'win32') {
      run('powershell', ['-NoProfile', '-Command', 'Expand-Archive', '-Force', '-Path', archive, '-DestinationPath', destDir]);
    } else {
      run('unzip', ['-o', archive, '-d', destDir]);
    }
  } else {
    run('tar', ['-xJf', archive, '-C', destDir]);
  }
}

function findExtractedBinary(dir, binName = BIN_NAME) {
  const direct = join(dir, binName);
  if (existsSync(direct)) return direct;

  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const candidate = join(dir, entry.name, binName);
    if (existsSync(candidate)) return candidate;
  }

  throw new Error(`Archive did not contain ${binName}`);
}

function installFromLocalBuild() {
  const localRelease = join(PACKAGE_ROOT, 'target', 'release', BIN_NAME);
  if (!existsSync(localRelease)) return false;
  mkdirSync(BIN_DIR, { recursive: true });
  copyFileSync(localRelease, DEST);
  chmodSync(DEST, 0o755);
  return true;
}

async function installFromRelease() {
  const target = cargoTarget();
  const archive = artifactName(target);
  const base = releaseBaseUrl();
  const tmpDir = join(BIN_DIR, '.tmp');
  const archivePath = join(tmpDir, archive);
  const checksumPath = `${archivePath}.sha256`;

  rmSync(tmpDir, { recursive: true, force: true });
  mkdirSync(tmpDir, { recursive: true });
  await download(`${base}/${archive}`, archivePath);
  await download(`${base}/${archive}.sha256`, checksumPath);
  verifyChecksum(archivePath, readFileSync(checksumPath, 'utf8'));
  extract(archivePath, tmpDir);

  const extracted = findExtractedBinary(tmpDir);
  mkdirSync(BIN_DIR, { recursive: true });
  copyFileSync(extracted, DEST);
  chmodSync(DEST, 0o755);
  rmSync(tmpDir, { recursive: true, force: true });
}

async function main() {
  if (installFromLocalBuild()) return;
  await installFromRelease();
}

if (require.main === module) {
  main().catch((error) => {
    console.error(error.message);
    process.exit(1);
  });
}

module.exports = {
  TARGETS,
  artifactName,
  cargoTarget,
  findExtractedBinary,
  platformKey,
  releaseBaseUrl,
  sha256,
  verifyChecksum,
};
