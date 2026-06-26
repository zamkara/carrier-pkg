use crate::config::Config;
use crate::container::exec_in_container;

fn list_pkg_files(config: &Config, pkg: &str) -> Result<Vec<String>, String> {
    let (cmd, args): (&str, Vec<&str>) = match config.pm.as_str() {
        "dnf" | "yum" | "zypper" => ("rpm", vec!["-ql", pkg]),
        "apt" | "apt-get" => ("dpkg", vec!["-L", pkg]),
        "pacman" => ("pacman", vec!["-Qlq", pkg]),
        "apk" => ("apk", vec!["info", "-L", pkg]),
        "xbps-install" => ("xbps-query", vec!["-f", pkg]),
        "emerge" => ("equery", vec!["files", pkg]),
        _ => return Ok(Vec::new()),
    };

    let output = exec_in_container(config, cmd, &args)?;
    Ok(output.lines().map(|l| l.to_string()).collect())
}

pub fn resolve_desktop_paths(config: &Config, pkg: &str) -> Result<Vec<String>, String> {
    let files = list_pkg_files(config, pkg)?;
    Ok(files
        .into_iter()
        .filter(|f| f.contains("/applications/") && f.ends_with(".desktop"))
        .collect())
}

pub fn resolve_desktop_ids(config: &Config, pkg: &str) -> Result<Vec<String>, String> {
    let paths = resolve_desktop_paths(config, pkg)?;
    Ok(paths
        .into_iter()
        .map(|p| {
            p.rsplit('/')
                .next()
                .unwrap_or(&p)
                .strip_suffix(".desktop")
                .unwrap_or("")
                .to_string()
        })
        .collect())
}

pub fn resolve_bin_paths(config: &Config, pkg: &str) -> Result<Vec<String>, String> {
    let files = list_pkg_files(config, pkg)?;
    Ok(files
        .into_iter()
        .filter(|f| {
            f.starts_with("/usr/bin/") || f.starts_with("/bin/")
        })
        .collect())
}
