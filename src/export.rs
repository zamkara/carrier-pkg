use crate::config::Config;
use crate::container::run_as_user;
use crate::file_resolve;

fn read_key() -> char {
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
    use std::io::Read;

    enable_raw_mode().ok();
    let mut buf = [0u8; 1];
    std::io::stdin().read_exact(&mut buf).ok();
    disable_raw_mode().ok();
    buf[0] as char
}

fn run_export(config: &Config, args: &[&str]) -> bool {
    let mut enter = run_as_user(config, "distrobox", &["enter", &config.container_name, "--", "distrobox-export"]);
    enter.args(args);
    // Silence output
    enter.stdout(std::process::Stdio::null());
    enter.stderr(std::process::Stdio::null());
    enter.status().map(|s| s.success()).unwrap_or(false)
}

pub fn prompt_export(config: &Config, pkg: &str) {
    let desktop_paths = file_resolve::resolve_desktop_paths(config, pkg).unwrap_or_default();
    let bin_paths = file_resolve::resolve_bin_paths(config, pkg).unwrap_or_default();

    println!();
    println!("Export '{}'?", pkg);
    println!("\x1b[31mN\x1b[0m Nope  \x1b[32mA\x1b[0m App  \x1b[32mB\x1b[0m Only Binary executable");

    let reply = read_key();
    println!();

    let mut succeeded = false;

    match reply {
        'a' | 'A' => {
            for path in &desktop_paths {
                if run_export(config, &["--app", path]) {
                    succeeded = true;
                }
            }
            if !succeeded && run_export(config, &["--app", pkg]) {
                succeeded = true;
            }
            if !succeeded {
                for path in &bin_paths {
                    if run_export(config, &["--bin", path]) {
                        succeeded = true;
                    }
                }
            }
            if !succeeded {
                let fallback = format!("/usr/bin/{}", pkg);
                if run_export(config, &["--bin", &fallback]) {
                    succeeded = true;
                }
            }
        }
        'b' | 'B' => {
            for path in &bin_paths {
                if run_export(config, &["--bin", path]) {
                    succeeded = true;
                }
            }
            if !succeeded {
                let fallback = format!("/usr/bin/{}", pkg);
                if run_export(config, &["--bin", &fallback]) {
                    succeeded = true;
                }
            }
        }
        _ => {
            println!("skipped");
            return;
        }
    }

    if succeeded {
        println!("\u{2714} installed {}", pkg);
    } else {
        println!("installed {}, \u{2717} export failed", pkg);
    }
}

pub fn cleanup_export(config: &Config, pkg: &str) {
    let desktop_paths = file_resolve::resolve_desktop_paths(config, pkg).unwrap_or_default();
    let desktop_ids = file_resolve::resolve_desktop_ids(config, pkg).unwrap_or_default();
    let bin_paths = file_resolve::resolve_bin_paths(config, pkg).unwrap_or_default();

    for path in &desktop_paths {
        run_export(config, &["--app", path, "-d"]);
    }
    run_export(config, &["--app", pkg, "-d"]);
    for path in &bin_paths {
        run_export(config, &["--bin", path, "-d"]);
    }
    let fallback = format!("/usr/bin/{}", pkg);
    run_export(config, &["--bin", &fallback, "-d"]);

    // Force-remove host-side desktop files
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let appdir = format!("{}/.local/share/applications", home);

    for id in &desktop_ids {
        let f = format!("{}/{}-{}.desktop", appdir, config.container_name, id);
        let _ = std::fs::remove_file(&f);
    }

    // Brute-force any desktop file containing pkg name
    if let Ok(entries) = std::fs::read_dir(&appdir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(&format!("{}-", config.container_name))
                && name_str.contains(pkg)
                && name_str.ends_with(".desktop")
            {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    println!("\u{2714} cleaned {}", pkg);
}
