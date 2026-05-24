use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::Write;
use std::collections::HashSet;
use chrono::Local;
use inquire::{Confirm, Password, Select, Text};

#[derive(Debug, PartialEq)]
pub enum SystemMode {
    LiveInstaller,
    Coexistence,
    Bootstrap,
}

impl std::fmt::Display for SystemMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemMode::LiveInstaller => write!(f, "Bare-Metal / VM Live Installer"),
            SystemMode::Coexistence => write!(f, "Dual-DE Coexistence (alongside other Desktop Environments)"),
            SystemMode::Bootstrap => write!(f, "Clean Minimal Fedora Conversion"),
        }
    }
}

fn detect_system_mode() -> SystemMode {
    if Path::new("/run/initramfs/live").exists() || std::env::var("OMARA_LIVE_MOCK").is_ok() {
        SystemMode::LiveInstaller
    } else if has_other_des() {
        SystemMode::Coexistence
    } else {
        SystemMode::Bootstrap
    }
}

fn has_other_des() -> bool {
    let mut other_found = false;
    for dir in &["/usr/share/wayland-sessions", "/usr/share/xsessions"] {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".desktop") && !name.contains("niri") {
                        other_found = true;
                        break;
                    }
                }
            }
        }
    }
    other_found
}

fn get_available_disks() -> Vec<String> {
    let output = Command::new("lsblk")
        .args(["-dno", "NAME,SIZE,MODEL"])
        .output();
    
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut disks = Vec::new();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                disks.push(trimmed.to_string());
            }
        }
        if !disks.is_empty() {
            return disks;
        }
    }
    
    vec![
        "sda (250GB, Virtual Disk)".to_string(),
        "nvme0n1 (512GB, NVMe Virtual Disk)".to_string(),
    ]
}

fn get_offline_image_path() -> Option<PathBuf> {
    let paths = [
        "/run/initramfs/live/omara-base.tar.zst",
        "/run/initramfs/live/LiveOS/rootfs.img",
        "/tmp/omara-base.tar.zst",
    ];
    for p in &paths {
        let path = Path::new(p);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }
    None
}

fn parse_manifest_file(file_path: &Path, manifest_dir: &Path, visited: &mut HashSet<PathBuf>, apps: &mut Vec<String>) {
    let canonical = match fs::canonicalize(file_path) {
        Ok(path) => path,
        Err(_) => file_path.to_path_buf(),
    };
    if !visited.insert(canonical) {
        return;
    }

    if let Ok(content) = fs::read_to_string(file_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with("@include ") {
                let include_path = trimmed["@include ".len()..].trim();
                let full_include_path = manifest_dir.join(include_path);
                parse_manifest_file(&full_include_path, manifest_dir, visited, apps);
            } else if !trimmed.starts_with('@') {
                apps.push(trimmed.to_string());
            }
        }
    }
}

pub fn load_app_manifests() -> Vec<String> {
    let manifest_dir = crate::paths::get_component_path("omara-os").join("manifests").join("dnf");
    let default_path = manifest_dir.join("default.txt");
    let mut apps = Vec::new();
    let mut visited = HashSet::new();
    
    if default_path.exists() {
        parse_manifest_file(&default_path, &manifest_dir, &mut visited, &mut apps);
        if !apps.is_empty() {
            return apps;
        }
    }
    
    vec![
        "firefox".to_string(),
        "kitty".to_string(),
        "fish".to_string(),
        "neovim".to_string(),
        "fastfetch".to_string(),
        "htop".to_string(),
    ]
}

fn install_system_packages() {
    let apps = load_app_manifests();
    if apps.is_empty() {
        println!("  ⚠️  No packages found in manifests, skipping package install.");
        return;
    }
    println!("  Installing default package set ({} apps)...", apps.len());
    
    let mut cmd = Command::new("sudo");
    cmd.arg("dnf").arg("install").arg("-y");
    for app in &apps {
        cmd.arg(app);
    }
    
    let status = cmd.status();
    if status.map(|s| s.success()).unwrap_or(false) {
        println!("  ✅ Installed all packages.");
    } else {
        eprintln!("  ❌ Package installation encountered errors.");
    }
}

fn install_packages_to_root(root: &str) {
    let apps = load_app_manifests();
    if apps.is_empty() {
        println!("  ⚠️  No packages found in manifests, skipping target package install.");
        return;
    }
    println!("  Installing default package set into target root ({} apps)...", apps.len());
    
    let mut cmd = Command::new("sudo");
    cmd.arg("dnf").arg(format!("--installroot={}", root)).arg("install").arg("-y");
    for app in &apps {
        cmd.arg(app);
    }
    
    let status = cmd.status();
    if status.map(|s| s.success()).unwrap_or(false) {
        println!("  ✅ Installed all packages into target root.");
    } else {
        eprintln!("  ❌ Package installation in target root encountered errors.");
    }
}

fn get_partition_path(disk_path: &str, index: usize) -> String {
    if disk_path.chars().last().map_or(false, |c| c.is_ascii_digit()) {
        format!("{}p{}", disk_path, index)
    } else {
        format!("{}{}", disk_path, index)
    }
}

fn get_partition_uuid(partition_path: &str) -> Option<String> {
    let output = Command::new("sudo")
        .args(["blkid", "-s", "UUID", "-o", "value", partition_path])
        .output();
    
    if let Ok(out) = output {
        let uuid = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !uuid.is_empty() {
            return Some(uuid);
        }
    }
    None
}

fn write_file_as_sudo(path: &str, content: &str) -> anyhow::Result<()> {
    let temp_path = format!("/tmp/omara_temp_{}", Local::now().format("%H%M%S%f"));
    fs::write(&temp_path, content)?;
    let status = Command::new("sudo")
        .args(["cp", &temp_path, path])
        .status()?;
    let _ = fs::remove_file(&temp_path);
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Failed to write to file {} as sudo", path)
    }
}

fn get_host_releasever() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("VERSION_ID=") {
                return line.trim_start_matches("VERSION_ID=")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }
    "44".to_string()
}

pub fn run_install(force: bool, dry_run: bool) -> anyhow::Result<()> {
    let mode = detect_system_mode();
    println!("{}", "🖥️  Omara OS Installer".bold().cyan());
    println!("  Detected Mode: {}", mode.to_string().yellow());
    println!();

    match mode {
        SystemMode::LiveInstaller => {
            println!("{}", "🚀 Starting Bare-Metal Interactive Installer...".bold().cyan());
            
            let disks = get_available_disks();
            let selected_disk = Select::new("Select target disk for Omara OS installation:", disks).prompt()?;
            
            let partition_options = vec![
                "Automatic Partitioning (Erase entire disk)",
                "Manual Partitioning (Launch cfdisk)",
            ];
            let partition_choice = Select::new("Select partitioning method:", partition_options).prompt()?;
            
            let hostname = Text::new("Enter hostname:").with_default("omara").prompt()?;
            let username = Text::new("Enter username:").with_default("jeryd").prompt()?;
            
            let mut password;
            loop {
                password = Password::new("Enter password:")
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .prompt()?;
                let confirm = Password::new("Confirm password:")
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .prompt()?;
                
                if password == confirm {
                    break;
                }
                println!("{}", "❌ Passwords do not match. Please try again.".red());
            }

            println!();
            println!("{}", "📋 Installation Summary".bold().cyan());
            println!("  Target Disk:   {}", selected_disk.yellow());
            println!("  Partitioning:  {}", partition_choice.yellow());
            println!("  Hostname:      {}", hostname.yellow());
            println!("  Username:      {}", username.yellow());
            println!();

            if dry_run {
                println!("{}", "⚠️  DRY-RUN MODE — No changes will be written.".yellow().bold());
                println!("Planned Actions (Simulated):");
                let disk_name = selected_disk.split_whitespace().next().unwrap_or("sda");
                let disk_path = format!("/dev/{}", disk_name);
                let p_boot = get_partition_path(&disk_path, 1);
                let p_root = get_partition_path(&disk_path, 2);
                println!("  1. [Dry Run] Wipe disk signatures on {}", disk_path);
                if partition_choice.contains("Manual") {
                    println!("  2. [Dry Run] Would launch cfdisk for {}", disk_path);
                } else {
                    println!("  2. [Dry Run] Create partition table on {} (EFI size 1GiB, Root remaining)", disk_path);
                    println!("  3. [Dry Run] Format EFI partition {} as FAT32", p_boot);
                    println!("  4. [Dry Run] Format root partition {} as ext4", p_root);
                }
                println!("  5. [Dry Run] Mount root partition {} to /mnt/sysroot", p_root);
                println!("  6. [Dry Run] Mount EFI partition {} to /mnt/sysroot/boot/efi", p_boot);
                if let Some(image_path) = get_offline_image_path() {
                    println!("  7. [Dry Run] Extract offline base system image ({}) to /mnt/sysroot", image_path.display());
                } else {
                    println!("  7. [Dry Run] Run online DNF bootstrap to /mnt/sysroot (release version: {})", get_host_releasever());
                }
                println!("  8. [Dry Run] Bind mount /dev, /proc, /sys to /mnt/sysroot");
                println!("  9. [Dry Run] Generate /etc/fstab and /etc/hostname (hostname: {})", hostname);
                println!(" 10. [Dry Run] Configure user account '{}' with sudo privileges", username);
                println!(" 11. [Dry Run] Run grub2-mkconfig and efibootmgr inside chroot");
                println!(" 12. [Dry Run] Clean up and unmount bind mounts and target partitions");
                println!();
                println!("✅ Dry-run simulation completed successfully.");
                return Ok(());
            }

            let proceed = Confirm::new("Proceed with installation? This will format the selected disk and erase all data.")
                .with_default(false)
                .prompt()?;

            if !proceed {
                println!("Installation aborted.");
                return Ok(());
            }

            println!("Checking root privileges (may prompt for sudo password)...");
            let sudo_check = Command::new("sudo").arg("true").status();
            if !sudo_check.map(|s| s.success()).unwrap_or(false) {
                anyhow::bail!("This installation requires root/sudo privileges to proceed.");
            }

            println!("{}", "⌛ Running installation...".bold().cyan());
            let disk_name = selected_disk.split_whitespace().next().unwrap_or("sda");
            let disk_path = format!("/dev/{}", disk_name);
            let boot_partition = get_partition_path(&disk_path, 1);
            let root_partition = get_partition_path(&disk_path, 2);
            
            if partition_choice.contains("Manual") {
                println!("  Launching cfdisk for {}...", disk_path);
                let _ = Command::new("sudo").arg("cfdisk").arg(&disk_path).status();
            } else {
                println!("  Wiping signatures on {}...", disk_path);
                let _ = Command::new("sudo").args(["wipefs", "-a", &disk_path]).status();

                println!("  Creating new GPT partition table on {}...", disk_path);
                let sfdisk_input = "label: gpt\nsize=1GiB, type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B\ntype=0FC63DAF-8483-4772-8E79-3D69D8477DE4\n";
                let mut child = Command::new("sudo")
                    .args(["sfdisk", &disk_path])
                    .stdin(std::process::Stdio::piped())
                    .spawn()?;
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(sfdisk_input.as_bytes())?;
                }
                let status = child.wait()?;
                if !status.success() {
                    anyhow::bail!("Failed to partition target disk {}", disk_path);
                }

                let _ = Command::new("sudo").args(["partprobe", &disk_path]).status();
                let _ = Command::new("sudo").args(["udevadm", "settle"]).status();

                println!("  Formatting EFI partition ({}) as FAT32...", boot_partition);
                let _ = Command::new("sudo").args(["mkfs.vfat", "-F32", &boot_partition]).status();

                println!("  Formatting Root partition ({}) as ext4...", root_partition);
                let _ = Command::new("sudo").args(["mkfs.ext4", "-F", &root_partition]).status();
            }
            
            let sysroot = "/mnt/sysroot";
            println!("  Mounting target root to {}...", sysroot);
            let _ = fs::create_dir_all(sysroot);
            
            let mount_root_status = Command::new("sudo")
                .args(["mount", &root_partition, sysroot])
                .status();
            
            if mount_root_status.map(|s| s.success()).unwrap_or(false) {
                let efi_dir = format!("{}/boot/efi", sysroot);
                let _ = Command::new("sudo").args(["mkdir", "-p", &efi_dir]).status();
                let mount_boot_status = Command::new("sudo").args(["mount", &boot_partition, &efi_dir]).status();
                if !mount_boot_status.map(|s| s.success()).unwrap_or(false) {
                    let _ = Command::new("sudo").args(["umount", sysroot]).status();
                    anyhow::bail!("Failed to mount EFI partition to {}", efi_dir);
                }
                
                let mut copy_success = false;
                if let Some(image_path) = get_offline_image_path() {
                    println!("  📦 Found offline system image at: {}", image_path.display().to_string().yellow());
                    if image_path.extension().map_or(false, |ext| ext == "img") {
                        println!("  → Mounting and copying rootfs.img contents...");
                        let temp_mount = "/mnt/live_rootfs";
                        let _ = Command::new("sudo").args(["mkdir", "-p", temp_mount]).status();
                        let mount_img = Command::new("sudo")
                            .args(["mount", "-o", "loop,ro", image_path.to_str().unwrap(), temp_mount])
                            .status();
                        
                        if mount_img.map(|s| s.success()).unwrap_or(false) {
                            println!("    Copying system files to target...");
                            let cp_status = Command::new("sudo")
                                .args(["cp", "-a", &format!("{}/.", temp_mount), sysroot])
                                .status();
                            
                            let _ = Command::new("sudo").args(["umount", temp_mount]).status();
                            
                            if cp_status.map(|s| s.success()).unwrap_or(false) {
                                copy_success = true;
                            } else {
                                eprintln!("  ❌ Failed to copy rootfs.img contents. Attempting DNF bootstrap fallback...");
                            }
                        } else {
                            eprintln!("  ❌ Failed to mount rootfs.img. Attempting DNF bootstrap fallback...");
                        }
                    } else {
                        println!("  → Extracting base OS files...");
                        let extract_status = Command::new("sudo")
                            .args(["tar", "--zstd", "-xpf", image_path.to_str().unwrap(), "-C", sysroot])
                            .status();
                        
                        if extract_status.map(|s| s.success()).unwrap_or(false) {
                            copy_success = true;
                        } else {
                            eprintln!("  ❌ Failed to extract offline image. Attempting DNF bootstrap fallback...");
                        }
                    }
                }

                if !copy_success {
                    println!("  🌐 Bootstrapping OS online via DNF...");
                    let target_repos_dir = format!("{}/etc/yum.repos.d", sysroot);
                    let target_pki_dir = format!("{}/etc/pki", sysroot);
                    let _ = Command::new("sudo").args(["mkdir", "-p", &target_repos_dir]).status();
                    let _ = Command::new("sudo").args(["mkdir", "-p", &target_pki_dir]).status();
                    let _ = Command::new("sudo").args(["cp", "-r", "/etc/yum.repos.d/.", &target_repos_dir]).status();
                    let _ = Command::new("sudo").args(["cp", "-r", "/etc/pki/.", &target_pki_dir]).status();

                    let releasever = get_host_releasever();
                    let dnf_status = Command::new("sudo")
                        .args(["dnf", "--installroot=/mnt/sysroot", &format!("--releasever={}", releasever), "groupinstall", "-y", "Core", "Standard"])
                        .status();
                    
                    if dnf_status.map(|s| s.success()).unwrap_or(false) {
                        install_packages_to_root(sysroot);
                    } else {
                        let _ = Command::new("sudo").args(["umount", &efi_dir]).status();
                        let _ = Command::new("sudo").args(["umount", sysroot]).status();
                        anyhow::bail!("Failed to run DNF bootstrap to target root");
                    }
                }
                
                println!("  → Binding system directories for chroot configuration...");
                let mounts = [
                    ("/dev", "dev"),
                    ("/proc", "proc"),
                    ("/sys", "sys"),
                    ("/sys/firmware/efi/efivars", "sys/firmware/efi/efivars"),
                ];
                let mut mounted_paths = Vec::new();
                for &(host_path, guest_rel) in &mounts {
                    let target_path = format!("{}/{}", sysroot, guest_rel);
                    if Path::new(host_path).exists() {
                        let _ = fs::create_dir_all(&target_path);
                        let status = Command::new("sudo")
                            .args(["mount", "--bind", host_path, &target_path])
                            .status();
                        if status.map(|s| s.success()).unwrap_or(false) {
                            mounted_paths.push(target_path);
                        }
                    }
                }

                println!("  → Generating /etc/fstab...");
                let root_uuid = get_partition_uuid(&root_partition);
                let boot_uuid = get_partition_uuid(&boot_partition);
                
                let root_spec = root_uuid
                    .as_ref()
                    .map(|u| format!("UUID={}", u))
                    .unwrap_or_else(|| root_partition.clone());
                let boot_spec = boot_uuid
                    .as_ref()
                    .map(|u| format!("UUID={}", u))
                    .unwrap_or_else(|| boot_partition.clone());
                
                let fstab_content = format!(
                    "{} / ext4 defaults 1 1\n{} /boot/efi vfat defaults 0 2\n",
                    root_spec, boot_spec
                );
                let fstab_path = format!("{}/etc/fstab", sysroot);
                if let Err(e) = write_file_as_sudo(&fstab_path, &fstab_content) {
                    eprintln!("  ⚠️ Warning: Failed to write fstab: {}", e);
                }

                println!("  → Setting hostname to '{}'...", hostname);
                let hostname_path = format!("{}/etc/hostname", sysroot);
                if let Err(e) = write_file_as_sudo(&hostname_path, &hostname) {
                    eprintln!("  ⚠️ Warning: Failed to write hostname: {}", e);
                }

                println!("  → Configuring administrator account...");
                let user_status = Command::new("sudo")
                    .args(["chroot", sysroot, "useradd", "-m", "-G", "wheel", &username])
                    .status();

                if user_status.map(|s| s.success()).unwrap_or(false) {
                    let passwd_input = format!("{}:{}", username, password);
                    let mut child = Command::new("sudo")
                        .args(["chroot", sysroot, "chpasswd"])
                        .stdin(std::process::Stdio::piped())
                        .spawn()?;
                    
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(passwd_input.as_bytes());
                    }
                    let _ = child.wait();

                    println!("  → Setting up sudo for wheel group...");
                    let sudoers_file = format!("{}/etc/sudoers.d/wheel", sysroot);
                    let _ = write_file_as_sudo(&sudoers_file, "%wheel ALL=(ALL) ALL\n");
                    let _ = Command::new("sudo").args(["chmod", "0440", &sudoers_file]).status();
                } else {
                    eprintln!("  ⚠️ Warning: Failed to add user '{}' via chroot.", username);
                }

                println!("  → Installing and configuring GRUB bootloader...");
                let grub_status = Command::new("sudo")
                    .args(["chroot", sysroot, "grub2-mkconfig", "-o", "/boot/grub2/grub.cfg"])
                    .status();
                if !grub_status.map(|s| s.success()).unwrap_or(false) {
                    eprintln!("  ⚠️ Warning: grub2-mkconfig returned non-zero status.");
                }

                println!("  → Registering EFI boot entry...");
                let efiboot_status = Command::new("sudo")
                    .args([
                        "chroot",
                        sysroot,
                        "efibootmgr",
                        "-c",
                        "-d",
                        &disk_path,
                        "-p",
                        "1",
                        "-L",
                        "Omara OS",
                        "-l",
                        "\\EFI\\fedora\\shimx64.efi",
                    ])
                    .status();
                if !efiboot_status.map(|s| s.success()).unwrap_or(false) {
                    eprintln!("  ⚠️ Warning: efibootmgr command returned non-zero status.");
                }

                println!("  → Unmounting bind mounts...");
                for path in mounted_paths.iter().rev() {
                    let _ = Command::new("sudo").args(["umount", "-l", path]).status();
                }
                
                println!("  Cleaning up and unmounting target...");
                let _ = Command::new("sudo").args(["umount", &efi_dir]).status();
                let _ = Command::new("sudo").args(["umount", sysroot]).status();
                
                println!("  ✅ Installation completed successfully! Please reboot your machine.");
            } else {
                anyhow::bail!("Failed to mount target partitions.");
            }
        }
        SystemMode::Coexistence => {
            let proceed = if force {
                true
            } else {
                Confirm::new("GNOME/KDE detected. Install Omara DE packages alongside your existing setup?")
                    .with_default(true)
                    .prompt()?
            };

            if !proceed {
                println!("Installation aborted.");
                return Ok(());
            }

            if dry_run {
                println!("{}", "⚠️  DRY-RUN MODE — No changes will be written.".yellow().bold());
                println!("Planned Actions (Simulated):");
                println!("  1. [Dry Run] Enable custom repositories (Tailscale, RPM Fusion, etc.).");
                println!("  2. [Dry Run] Install DNF packages from omara-os manifests.");
                println!();
                println!("✅ Dry-run simulation completed successfully.");
                return Ok(());
            }

            println!("🚀 Running Coexistence Setup...");
            install_system_packages();
            println!("  ✅ GNOME setup registered.");
        }
        SystemMode::Bootstrap => {
            let proceed = if force {
                true
            } else {
                Confirm::new("Install Omara DE packages on this minimal Fedora system?")
                    .with_default(true)
                    .prompt()?
            };

            if !proceed {
                println!("Installation aborted.");
                return Ok(());
            }

            if dry_run {
                println!("{}", "⚠️  DRY-RUN MODE — No changes will be written.".yellow().bold());
                println!("Planned Actions (Simulated):");
                println!("  1. [Dry Run] Enable custom repositories.");
                println!("  2. [Dry Run] Install DNF packages from omara-os manifests.");
                println!();
                println!("✅ Dry-run simulation completed successfully.");
                return Ok(());
            }

            println!("🚀 Running Fedora Minimal Bootstrap...");
            install_system_packages();
            println!("  ✅ System bootstrap complete. Restart machine to boot into Omara.");
        }
    }
    Ok(())
}
