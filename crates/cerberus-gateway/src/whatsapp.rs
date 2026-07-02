use std::process::{Command, Child};
use std::path::Path;
use anyhow::{Result, Context};
use std::fs;

pub struct WhatsappBridge {
    process: Child,
}

impl WhatsappBridge {
    pub fn start<P: AsRef<Path>>(workspace_root: P) -> Result<Self> {
        let bridge_dir = workspace_root.as_ref().join("scripts").join("whatsapp-bridge");
        let session_dir = workspace_root.as_ref().join(".cerberus").join("whatsapp_session");
        
        fs::create_dir_all(&session_dir)?;
        
        // Ensure npm install is run
        if !bridge_dir.join("node_modules").exists() {
            println!("Installing WhatsApp bridge dependencies...");
            let status = Command::new(if cfg!(windows) { "npm.cmd" } else { "npm" })
                .current_dir(&bridge_dir)
                .args(["install", "--silent"])
                .status()
                .context("Failed to run npm install")?;
                
            if !status.success() {
                anyhow::bail!("npm install failed for whatsapp-bridge");
            }
        }
        
        println!("Starting WhatsApp Node.js bridge. Scan the QR code if prompted...");
        
        let process = Command::new(if cfg!(windows) { "node.exe" } else { "node" })
            .current_dir(&bridge_dir)
            .arg("bridge.js")
            .arg("--session")
            .arg(&session_dir)
            .spawn()
            .context("Failed to start Node.js bridge")?;
            
        Ok(Self { process })
    }
    
    pub fn stop(&mut self) -> Result<()> {
        self.process.kill()?;
        self.process.wait()?;
        Ok(())
    }
}
