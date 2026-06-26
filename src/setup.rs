use std::fs;
use std::os::unix::fs::symlink;

const ARK_DIR: &str = "/usr/lib/ark";
const BIN_DIR: &str = "/usr/local/bin";

const SUPPORTED_PMS: &[&str] = &[
    "pacman", "apt-get", "apt", "dnf", "yum", "zypper", "apk", "emerge", "xbps-install",
    "slackpkg", "opkg",
];

pub fn install() -> Result<(), String> {
    println!("installing carrier...");

    fs::create_dir_all(ARK_DIR)
        .map_err(|e| format!("failed to create {}: {}", ARK_DIR, e))?;
    fs::create_dir_all(BIN_DIR)
        .map_err(|e| format!("failed to create {}: {}", BIN_DIR, e))?;

    let self_path = std::env::current_exe()
        .map_err(|e| format!("failed to get current exe path: {}", e))?;

    // Create symlinks
    for pm in SUPPORTED_PMS {
        let link = format!("{}/{}", BIN_DIR, pm);
        let _ = fs::remove_file(&link);
        symlink(&self_path, &link)
            .map_err(|e| format!("failed to symlink {}: {}", link, e))?;
    }

    println!("\u{2714} symlinks created ({})", SUPPORTED_PMS.join(" "));
    println!();
    println!("done. run e.g.: apk add firefox");

    Ok(())
}
