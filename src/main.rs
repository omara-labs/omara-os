use clap::{Parser, Subcommand};
use inquire::Select;
use colored::Colorize;

mod paths;
mod install;
mod reset;
mod config;

#[derive(Parser)]
#[command(
    name = "omara-os",
    version,
    about = "The official Omara OS TUI Installer & Administrative Setup utility",
    long_about = "Manage installations, system resets, and system-level environment variables."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the interactive OS installer
    Install {
        /// Force install without environment verification checks
        #[arg(short, long)]
        force: bool,

        /// Dry-run simulation mode
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Reset system configuration and packages to Omara defaults
    Reset {
        /// Proceed without backup or confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Interactively manage system and environment variables
    Config,
}

fn show_tui_dashboard() -> anyhow::Result<()> {
    println!("{}", "=============================================".cyan());
    println!("{}", "       ❄️  OMARA OS ADMINISTRATIVE TUI ❄️       ".bold().cyan());
    println!("{}", "=============================================".cyan());
    println!();

    let choices = vec![
        "1. Install / Bootstrap Omara OS",
        "2. Reset Configurations & Packages to Default",
        "3. Edit System & Environment Variables",
        "4. Exit",
    ];

    loop {
        let selection = Select::new("Choose an administrative action:", choices.clone()).prompt()?;

        match selection {
            "1. Install / Bootstrap Omara OS" => {
                let dry_run = inquire::Confirm::new("Run in dry-run mode?")
                    .with_default(false)
                    .prompt()?;
                let _ = install::run_install(false, dry_run);
            }
            "2. Reset Configurations & Packages to Default" => {
                let _ = reset::run_reset(false);
            }
            "3. Edit System & Environment Variables" => {
                let _ = config::run_variables_setup();
            }
            "4. Exit" | _ => {
                println!("Goodbye!");
                break;
            }
        }
        println!();
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Some(Commands::Install { force, dry_run }) => install::run_install(*force, *dry_run),
        Some(Commands::Reset { yes }) => reset::run_reset(*yes),
        Some(Commands::Config) => config::run_variables_setup(),
        None => show_tui_dashboard(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
