#[derive(Debug, PartialEq)]
pub enum Operation {
    Install,
    Remove,
    Upgrade,
    Search,
    Noop,
}

pub fn detect_operation(pm: &str, args: &[String]) -> Operation {
    match pm {
        "apt" | "apt-get" => {
            for arg in args {
                if arg.starts_with('-') {
                    continue;
                }
                match arg.as_str() {
                    "install" => return Operation::Install,
                    "remove" | "purge" | "autoremove" => return Operation::Remove,
                    "update" | "upgrade" | "full-upgrade" | "dist-upgrade" => return Operation::Upgrade,
                    "search" | "show" => return Operation::Search,
                    _ => return Operation::Install,
                }
            }
            Operation::Noop
        }
        "dnf" | "yum" => {
            for arg in args {
                if arg.starts_with('-') {
                    continue;
                }
                match arg.as_str() {
                    "install" | "groupinstall" | "localinstall" => return Operation::Install,
                    "remove" | "erase" | "autoremove" => return Operation::Remove,
                    "update" | "upgrade" | "distro-sync" | "check-update" => return Operation::Upgrade,
                    "search" | "provides" | "whatprovides" => return Operation::Search,
                    _ => return Operation::Noop,
                }
            }
            Operation::Noop
        }
        "pacman" => {
            let mut has_sync = false;
            let mut has_remove = false;
            let mut has_search = false;
            let mut has_query = false;
            let mut has_upgrade_file = false;

            for arg in args {
                match arg.as_str() {
                    "--noconfirm" | "--needed" | "--ask" | "--overwrite" | "--color"
                    | "--noprogressbar" | "--noscriptlet" | "--print" | "--quiet"
                    | "--verbose" | "--debug" | "--confirm" | "--disable-download-timeout"
                    | "--gpgdir" | "--keyserver" | "--print-format" | "--sysroot" | "--root"
                    | "--dbpath" | "--cachedir" | "--hookdir" | "--logfile" => continue,
                    "-S" | "--sync" => has_sync = true,
                    a if a.starts_with("-S") && a != "-S" => has_sync = true,
                    "-U" | "--upgrade" => has_upgrade_file = true,
                    a if a.starts_with("-U") && a != "-U" => has_upgrade_file = true,
                    a if a.starts_with("-R") => has_remove = true,
                    "--remove" => has_remove = true,
                    a if a.starts_with("-s") && a != "-s" => has_search = true,
                    "--search" => has_search = true,
                    "-Q" | "--query" => has_query = true,
                    a if a.starts_with("-Q") && a != "-Q" => has_query = true,
                    "-F" | "--files" => has_search = true,
                    _ => {}
                }
            }

            if has_search {
                return Operation::Search;
            }
            if has_query {
                return Operation::Noop;
            }
            if has_remove {
                return Operation::Remove;
            }
            if has_sync || has_upgrade_file {
                return Operation::Install;
            }
            Operation::Noop
        }
        "zypper" => {
            for arg in args {
                if arg.starts_with('-') {
                    continue;
                }
                match arg.as_str() {
                    "install" | "in" => return Operation::Install,
                    "remove" | "rm" => return Operation::Remove,
                    "update" | "up" | "dup" => return Operation::Upgrade,
                    "search" | "se" => return Operation::Search,
                    _ => return Operation::Noop,
                }
            }
            Operation::Noop
        }
        "apk" => {
            for arg in args {
                if arg.starts_with('-') {
                    continue;
                }
                match arg.as_str() {
                    "add" => return Operation::Install,
                    "del" => return Operation::Remove,
                    "update" | "upgrade" => return Operation::Upgrade,
                    "search" => return Operation::Search,
                    "info" | "list" => return Operation::Noop,
                    _ => return Operation::Noop,
                }
            }
            Operation::Noop
        }
        "emerge" => {
            for arg in args {
                if arg == "--unmerge" || arg == "-C" || arg == "--depclean" || arg == "-c" {
                    return Operation::Remove;
                }
                if arg == "--sync" {
                    return Operation::Upgrade;
                }
                if arg == "--search" || arg == "-s" || arg == "--pattern" {
                    return Operation::Search;
                }
            }
            // Any positional arg with emerge means install
            for arg in args {
                if !arg.starts_with('-') {
                    return Operation::Install;
                }
            }
            Operation::Noop
        }
        "xbps-install" => {
            let mut has_remove = false;
            let mut has_search = false;
            let mut has_sync_flag = false;
            let mut has_update_flag = false;
            let mut has_positional = false;

            for arg in args {
                match arg.as_str() {
                    "-r" | "--remove" => has_remove = true,
                    "-s" | "--search" => has_search = true,
                    "-S" | "--sync" => has_sync_flag = true,
                    "-u" | "--update" => has_update_flag = true,
                    a if a.starts_with("-Su") => {
                        has_sync_flag = true;
                        has_update_flag = true;
                    }
                    a if !a.starts_with('-') => has_positional = true,
                    _ => {}
                }
            }

            if has_search {
                return Operation::Search;
            }
            if has_remove {
                return Operation::Remove;
            }
            if has_positional {
                return Operation::Install;
            }
            if has_update_flag || has_sync_flag {
                return Operation::Upgrade;
            }
            Operation::Install
        }
        "slackpkg" | "opkg" => {
            for arg in args {
                if arg.starts_with('-') {
                    continue;
                }
                match arg.as_str() {
                    "install" | "reinstall" => return Operation::Install,
                    "remove" => return Operation::Remove,
                    "update" | "upgrade-all" | "upgrade" => return Operation::Upgrade,
                    "search" | "find" => return Operation::Search,
                    "info" | "list" | "list-installed" | "clean-system" | "generate" => return Operation::Noop,
                    _ => return Operation::Install,
                }
            }
            Operation::Noop
        }
        _ => Operation::Noop,
    }
}

pub fn extract_pkg_names(op: &Operation, pm: &str, args: &[String]) -> Vec<String> {
    let mut pkgs: Vec<String> = Vec::new();

    let is_install = *op == Operation::Install;
    let is_remove = *op == Operation::Remove;

    match pm {
        "xbps-install" | "emerge" => {
            for arg in args {
                if arg.is_empty() || arg.starts_with('-') {
                    continue;
                }
                pkgs.push(arg.clone());
            }
        }
        "pacman" => {
            let mut capturing = false;
            for arg in args {
                if arg.is_empty() {
                    continue;
                }
                if !capturing {
                    if is_install && (arg.starts_with("-S") || arg.starts_with("-U")
                        || arg == "--sync" || arg == "--upgrade")
                    {
                        capturing = true;
                    } else if is_remove && (arg.starts_with("-R") || arg == "--remove") {
                        capturing = true;
                    }
                    continue;
                }
                if arg == "--" || arg.starts_with('-') {
                    continue;
                }
                pkgs.push(arg.clone());
            }
        }
        _ => {
            let skip_subcmds: &[&str] = if is_install {
                &["install", "add", "in", "groupinstall", "localinstall", "--install", "reinstall"]
            } else if is_remove {
                &["remove", "purge", "autoremove", "erase", "del", "rm"]
            } else {
                &[]
            };

            let ignored: &[&str] = &[
                "install", "add", "reinstall", "remove", "del", "purge", "update", "upgrade",
                "full-upgrade", "dist-upgrade", "autoremove", "clean", "autoclean",
                "search", "find", "show", "query", "info", "profile", "--help", "--version",
                "-h", "-V", "-R", "-Q", "-U", "-F", "-s", "--remove", "--upgrade",
                "--query", "--search", "--info", "--clean", "in", "rm",
                "list", "list-installed", "clean-system", "generate", "upgrade-all",
            ];

            let mut found_subcmd = false;
            for arg in args {
                if arg.is_empty() {
                    continue;
                }
                if !found_subcmd {
                    if arg.starts_with('-') {
                        continue;
                    }
                    if skip_subcmds.contains(&arg.as_str()) {
                        found_subcmd = true;
                    }
                    continue;
                }
                if arg == "--" || arg.starts_with('-') {
                    continue;
                }
                if ignored.contains(&arg.as_str()) {
                    continue;
                }
                pkgs.push(arg.clone());
            }
        }
    }

    pkgs
}
