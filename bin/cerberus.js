#!/usr/bin/env node
'use strict';

const { spawnSync } = require('node:child_process');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const args = process.argv.slice(2);

const isWindows = process.platform === 'win32';
const binName = isWindows ? 'cerberus.exe' : 'cerberus';
const binPath = path.join(root, 'target', 'release', binName);

const result = spawnSync(binPath, args, {
  cwd: process.cwd(),
  stdio: 'inherit',
  shell: false,
});

if (result.error) {
  console.error(`cerberus: failed to launch Rust CLI: ${result.error.message}`);
  console.error(`Please ensure you ran 'npm install' so the postinstall script builds the binary.`);
  process.exit(1);
}

process.exit(result.status ?? 0);
