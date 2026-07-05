mod config;
mod container;
mod export;
mod file_resolve;
mod host_detect;
mod pm_detect;
mod setup;
mod spinner;

use config::Config;
use container::{ensure_container, run_in_container};
use export::{cleanup_export, prompt_export};
use pm_detect::{detect_operation, extract_pkg_names, Operation};

fn detect_pm_name() -> String {
    let arg0 = std::env::args().next().unwrap_or_default();

    if arg0.contains("setup") || arg0.contains("cleaner") {
        return "setup".to_string();
    }

    // Extract basename from path
    arg0.rsplit('/')
        .next()
        .unwrap_or(&arg0)
        .to_string()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pm_name = detect_pm_name();
    let is_carrier = pm_name == "carrier";

    // --version / --help only when called as "carrier" directly
    if is_carrier {
        for a in &args[1..] {
            if a == "--version" || a == "-V" {
                println!("carrier 0.1.0");
                return;
            }
            if a == "--help" || a == "-h" {
                println!("carrier - package manager wrapper via distrobox containers");
                println!();
                println!("Usage:");
                println!("  sudo <pm> <command>        run a package manager in a distrobox container");
                println!("  sudo carrier --setup        install symlinks");
                println!();
                println!("Supported PM symlinks:");
                println!("  pacman apt-get apt dnf yum zypper apk emerge xbps-install slackpkg opkg");
                println!();
                println!("Example:");
                println!("  sudo apk add firefox");
                return;
            }
        }
    }

    // When invoked as "carrier" directly (no PM symlink), --setup installs.
    if is_carrier {
        if args.get(1).map(|s| s.as_str()) == Some("--setup") {
            if let Err(e) = setup::install() {
                eprintln!("setup failed: {}", e);
                std::process::exit(1);
            }
            return;
        }
        eprintln!("carrier: invoke via a symlink named after a supported PM");
        eprintln!("  sudo carrier --help");
        std::process::exit(1);
    }

    let config = match Config::new(&pm_name) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("carrier: invoke via a symlink named after a supported PM");
            eprintln!("  sudo carrier --help");
            std::process::exit(1);
        }
    };

    // Setup mode (called via "setup" or "carrier-setup" symlink)
    if config.is_setup() {
        if let Err(e) = setup::install() {
            eprintln!("setup failed: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let rest_args: Vec<String> = args[1..].to_vec();

    let op = detect_operation(&config.pm, &rest_args);

    if !ensure_container(&config) {
        eprintln!("  carrier: container setup failed");
        std::process::exit(1);
    }

    // For remove: cleanup desktop files BEFORE package removal
    if op == Operation::Remove {
        let pkgs = extract_pkg_names(&op, &config.pm, &rest_args);
        for pkg in &pkgs {
            cleanup_export(&config, pkg);
        }
    }

    let rc = run_in_container(&config, &rest_args);

    // For install: prompt export after successful install
    if rc == 0 && op == Operation::Install {
        let pkgs = extract_pkg_names(&op, &config.pm, &rest_args);
        if !pkgs.is_empty() {
            if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
                println!("\n\u{2714} done");
            }
            for pkg in &pkgs {
                prompt_export(&config, pkg);
            }
        }
    }

    std::process::exit(rc);
}
