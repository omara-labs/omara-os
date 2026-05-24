use colored::Colorize;
use std::fs;
use std::path::Path;
use inquire::{Select, Text};

fn get_config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/jeryd".to_string());
    Path::new(&home).join(".config").join("omara").join("omara.conf")
}

pub fn run_variables_setup() -> anyhow::Result<()> {
    println!("{}", "⚙️  Omara OS System Variables & Configuration".bold().cyan());
    println!();

    let config_file = get_config_path();
    let mut session = "gnome".to_string();
    let mut workspace = "~/Projects/omara-labs".to_string();

    if config_file.exists() {
        if let Ok(content) = fs::read_to_string(&config_file) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("OMARA_SESSION=") {
                    session = trimmed["OMARA_SESSION=".len()..]
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                } else if trimmed.starts_with("OMARA_WORKSPACE_DIR=") {
                    workspace = trimmed["OMARA_WORKSPACE_DIR=".len()..]
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                }
            }
        }
    }

    println!("Current Settings:");
    println!("  OMARA_SESSION:       {}", session.yellow());
    println!("  OMARA_WORKSPACE_DIR: {}", workspace.yellow());
    println!();

    let choices = vec![
        "Change Target Session Environment (OMARA_SESSION)",
        "Change Workspace Directory (OMARA_WORKSPACE_DIR)",
        "Save and Exit",
        "Cancel",
    ];

    loop {
        let selection = Select::new("Select configuration action:", choices.clone()).prompt()?;

        match selection {
            "Change Target Session Environment (OMARA_SESSION)" => {
                let session_options = vec![
                    "gnome (Active / Recommended)",
                    "niri (Paused)",
                    "hyprland (Paused)",
                ];
                let chosen_session = Select::new("Choose session environment:", session_options).prompt()?;
                session = chosen_session.split_whitespace().next().unwrap_or("gnome").to_string();
                println!("Session environment set to {}", session.yellow());
            }
            "Change Workspace Directory (OMARA_WORKSPACE_DIR)" => {
                let new_dir = Text::new("Enter workspace path:")
                    .with_default(&workspace)
                    .prompt()?;
                workspace = new_dir;
                println!("Workspace directory path set to {}", workspace.yellow());
            }
            "Save and Exit" => {
                if let Some(parent) = config_file.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let config_content = format!(
                    "# Omara OS Configuration\nOMARA_SESSION=\"{}\"\nOMARA_WORKSPACE_DIR=\"{}\"\n",
                    session, workspace
                );
                fs::write(&config_file, config_content)?;
                println!("{}", "✅ Configuration saved successfully!".bold().green());
                break;
            }
            "Cancel" | _ => {
                println!("Configuration setup cancelled.");
                break;
            }
        }
        println!();
    }

    Ok(())
}
