use std::collections::HashMap;

pub struct Config {
    pub pm: String,
    pub image: String,
    pub container_name: String,
    pub sudo_uid: Option<u32>,
}

impl Config {
    pub fn new(pm: &str) -> Result<Self, String> {
        let image_map: HashMap<&str, &str> = [
            ("pacman", "ghcr.io/archlinux/archlinux:latest"),
            ("apt", "docker.io/library/debian:latest"),
            ("apt-get", "docker.io/library/debian:latest"),
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

        let container_map: HashMap<&str, &str> = [
            ("pacman", "archlinux"),
            ("apt", "debian"),
            ("apt-get", "debian"),
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
