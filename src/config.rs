use std::collections::HashMap;

pub struct Config {
    pub pm: String,
    pub image: String,
    pub container_name: String,
    pub sudo_uid: Option<u32>,
}

fn is_hardened_kernel() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|v| v.to_lowercase().contains("hardened"))
        .unwrap_or(false)
}

impl Config {
    pub fn new(pm: &str) -> Result<Self, String> {
        let hardened = is_hardened_kernel();

        let pacman_image = if hardened {
            "docker.io/blackarchlinux/blackarch:latest"
        } else {
            "ghcr.io/archlinux/archlinux:latest"
        };
        let debian_image = if hardened {
            "docker.io/parrotsec/core:latest"
        } else {
            "docker.io/library/debian:latest"
        };

        let image_map: HashMap<&str, &str> = [
            ("pacman", pacman_image),
            ("apt", debian_image),
            ("apt-get", debian_image),
            ("dnf", "docker.io/library/fedora:latest"),
            ("yum", "docker.io/library/fedora:latest"),
            ("zypper", "docker.io/opensuse/tumbleweed:latest"),
            ("apk", "docker.io/library/alpine:latest"),
            ("emerge", "docker.io/gentoo/stage3:latest"),
            ("xbps-install", "ghcr.io/void-linux/void-musl-full:latest"),
            (
                "slackpkg",
                "registry.slackware.nl/slackware/slackware:latest",
            ),
            ("opkg", "docker.io/openwrt/rootfs:latest"),
        ]
        .into_iter()
        .collect();

        let pacman_container = if hardened { "blackarch" } else { "archlinux" };
        let debian_container = if hardened { "kali" } else { "debian" };

        let container_map: HashMap<&str, &str> = [
            ("pacman", pacman_container),
            ("apt", debian_container),
            ("apt-get", debian_container),
            ("dnf", "fedora"),
            ("yum", "fedora"),
            ("zypper", "opensuse"),
            ("apk", "alpine"),
            ("emerge", "gentoo"),
            ("xbps-install", "void"),
            ("slackpkg", "slackware"),
            ("opkg", "openwrt"),
        ]
        .into_iter()
        .collect();

        let image = image_map
            .get(pm)
            .ok_or_else(|| format!("carrier: unknown package manager '{}'", pm))?;
        let container_name = container_map
            .get(pm)
            .ok_or_else(|| format!("carrier: unknown package manager '{}'", pm))?;

        let sudo_uid = std::env::var("SUDO_UID")
            .ok()
            .and_then(|v| v.parse::<u32>().ok());

        Ok(Config {
            pm: pm.to_string(),
            image: image.to_string(),
            container_name: container_name.to_string(),
            sudo_uid,
        })
    }

    pub fn is_setup(&self) -> bool {
        self.pm == "setup" || self.pm == "carrier-setup"
    }
}
