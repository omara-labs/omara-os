use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;
use chrono::Local;
use inquire::Confirm;

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn audit_and_install_packages() {
    let apps = crate::install::load_app_manifests();
    if apps.is_empty() {
        println!("    ⚠️  No packages found in manifests, skipping audit.");
        return;
    }
    
    let mut missing_apps = Vec::new();
    for app in &apps {
        if !command_exists(app) {
            missing_apps.push(app.clone());
        }
    }

    if missing_apps.is_empty() {
        println!("    All default packages are already installed.");
    } else {
        println!("    Missing packages detected: {}", missing_apps.join(", "));
        println!("    Installing missing packages...");
        
        let mut cmd = Command::new("sudo");
        cmd.arg("dnf").arg("install").arg("-y");
        for app in &missing_apps {
            cmd.arg(app);
        }
        let _ = cmd.status();
    }
}

pub fn run_reset(yes: bool) -> anyhow::Result<()> {
    println!("{}", "🔄  Omara System Reset".bold().cyan());
    println!("This will backup your existing config folder and restore defaults.");
    
    if !yes {
        let proceed = Confirm::new("Are you sure you want to proceed?")
            .with_default(false)
            .prompt()?;
        if !proceed {
            println!("Reset aborted.");
            return Ok(());
        }
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/jeryd".to_string());
    let config_dir = Path::new(&home).join(".config");

    let backup_suffix = Local::now().format("%Y%m%d%H%M%S").to_string();
    let components_to_reset = ["kitty", "fish", "gh", "gnome"];
    for component in &components_to_reset {
        let path = config_dir.join(component);
        if path.exists() {
            let backup_path = config_dir.join(format!("{}.bak.{}", component, backup_suffix));
            println!("  Backup: moving {} to {}", path.display(), backup_path.display());
            if let Err(e) = fs::rename(&path, &backup_path) {
                eprintln!("  ❌ Failed to backup config: {}", e);
            }
        }
    }

    let omara_configs_dir = crate::paths::get_component_path("omara-core");
    if omara_configs_dir.exists() {
        println!("  Restoring default templates from omara-configs...");
        let source_configs = omara_configs_dir.join("configs");
        if source_configs.exists() {
            for component in &components_to_reset {
                let src = source_configs.join(component);
                let dest = config_dir.join(component);
                if src.exists() {
                    println!("    Restoring default config: {}", dest.display());
                    if let Err(e) = copy_dir_all(&src, &dest) {
                        eprintln!("    ❌ Error restoring {}: {}", component, e);
                    }
                }
            }
        }
    } else {
        println!("  ⚠️  omara-configs not found at {}, skipping config restore.", omara_configs_dir.display());
    }

    println!("  Auditing package manifests...");
    audit_and_install_packages();

    println!("  ✅ System reset completed successfully!");
    Ok(())
}
