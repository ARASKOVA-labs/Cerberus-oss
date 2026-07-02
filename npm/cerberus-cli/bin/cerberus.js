#!/usr/bin/env node

const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const ext = process.platform === 'win32' ? '.exe' : '';
const binPath = path.join(__dirname, `cerberus${ext}`);

if (!fs.existsSync(binPath)) {
  // During local development, the binary isn't downloaded by install.js yet.
  // We'll fallback to running `cargo run` from the project root if it exists.
  const rootDir = path.resolve(__dirname, '../../../');
  const cargoToml = path.join(rootDir, 'Cargo.toml');
  
  if (fs.existsSync(cargoToml)) {
    console.warn(`[Cerberus] Native binary not found at ${binPath}. Falling back to 'cargo run' for local dev...`);
    const args = ['run', '-p', 'cerberus-cli', '--', ...process.argv.slice(2)];
    const result = spawnSync('cargo', args, { stdio: 'inherit', cwd: rootDir });
    process.exit(result.status ?? 1);
  } else {
    console.error(`[Cerberus] Fatal: Native binary not found at ${binPath}`);
    console.error(`[Cerberus] Please reinstall the package to download the correct binary for your OS.`);
    process.exit(1);
  }
}

const args = process.argv.slice(2);
const result = spawnSync(binPath, args, { stdio: 'inherit' });
process.exit(result.status ?? 1);
