#!/usr/bin/env node
'use strict';

const { spawnSync } = require('node:child_process');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const args = process.argv.slice(2);

const cargoArgs = ['run', '-q', '-p', 'cerberus-cli', '--', ...args];
const command = process.platform === 'win32' ? 'cmd.exe' : 'cargo';
const commandArgs = process.platform === 'win32'
  ? ['/d', '/s', '/c', 'cargo', ...cargoArgs]
  : cargoArgs;

const result = spawnSync(command, commandArgs, {
  cwd: root,
  stdio: 'inherit',
  shell: false,
});

if (result.error) {
  console.error(`cerberus: failed to launch Rust CLI: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 0);
