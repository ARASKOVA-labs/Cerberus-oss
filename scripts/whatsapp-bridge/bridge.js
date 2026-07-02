const { Client, LocalAuth } = require('whatsapp-web.js');
const qrcode = require('qrcode-terminal');
const path = require('path');
const fs = require('fs');

// Parse session directory from args
const args = process.argv.slice(2);
let sessionDir = './.wwebjs_auth';
if (args[0] === '--session' && args[1]) {
    sessionDir = args[1];
}

console.log(`Starting WhatsApp Bridge. Session dir: ${sessionDir}`);

const client = new Client({
    authStrategy: new LocalAuth({ dataPath: sessionDir }),
    puppeteer: {
        args: ['--no-sandbox', '--disable-setuid-sandbox'],
    }
});

client.on('qr', (qr) => {
    // Generate and scan this code with your phone
    console.log('\n[Cerberus] Scan the QR code below to connect WhatsApp:');
    qrcode.generate(qr, { small: true });
});

client.on('ready', () => {
    console.log('[Cerberus] WhatsApp Gateway is READY and connected.');
});

client.on('message', async msg => {
    // Basic echo for now to demonstrate connectivity
    // In production, this would forward to Cerberus state.db via Rust IPC or SQLite polling
    if (msg.body.startsWith('!cerberus ')) {
        const cmd = msg.body.substring(10);
        console.log(`[Cerberus] Received WhatsApp Command: ${cmd}`);
        msg.reply(`Cerberus received your command: ${cmd}`);
    }
});

client.on('auth_failure', msg => {
    console.error('[Cerberus] Authentication failure', msg);
});

client.initialize();
