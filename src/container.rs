use crate::config::Config;
use crate::spinner::spin_run;
use std::ffi::OsStr;
use std::process::Command;

pub fn run_as_user(config: &Config, program: &str, args: &[impl AsRef<OsStr>]) -> Command {
    if let Some(uid) = config.sudo_uid {
        let mut cmd = Command::new("systemd-run");
        cmd.arg("--user");
        cmd.arg(format!("-M{}@.host", uid));
        cmd.args(["--wait", "--collect", "--pipe", "--quiet", "--"]);
        cmd.arg(program);
        cmd.args(args);
        cmd
    } else {
        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd
    }
}

fn container_exists(config: &Config) -> bool {
    let mut cmd = run_as_user(config, "distrobox", &["list"]);
    let output = match cmd.output() {
        Ok(o) => o,
        Err(_) => return false,
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|line| line.contains(&config.container_name))
}

fn pull_image(config: &Config) -> bool {
    let runtime = if Command::new("podman").arg("--version").output().is_ok() {
        "podman"
    } else if Command::new("docker").arg("--version").output().is_ok() {
        "docker"
    } else {
        return true;
    };

    let mut check = run_as_user(config, runtime, &["image", "exists", &config.image]);
    if check.status().map(|s| s.success()).unwrap_or(false) {
        return true;
    }

    let mut pull = run_as_user(config, runtime, &["pull", &config.image]);
    spin_run("pulling image", &mut pull)
}

fn create_container(config: &Config) -> bool {
    let mut args: Vec<&str> = vec![
        "create",
        "--image",
        &config.image,
        "--name",
        &config.container_name,
    ];

    if config.pm == "apt" || config.pm == "apt-get" {
        let pre_init = "mkdir -p /etc/apt/apt.conf.d && printf 'Dpkg::Use-Pty \"0\";\\n' > /etc/apt/apt.conf.d/99-no-pty";
        let add_flags = "--env DEBIAN_FRONTEND=noninteractive";
        args.push("--pre-init-hooks");
        args.push(pre_init);
        args.push("--additional-flags");
        args.push(add_flags);
    }

    let mut create = run_as_user(config, "distrobox", &args);
    spin_run("preparing container", &mut create)
}

fn init_container(config: &Config) -> bool {
    let mut enter = run_as_user(config, "distrobox", &["enter", &config.container_name, "--", "true"]);
    spin_run("initializing container", &mut enter)
}

pub fn ensure_container(config: &Config) -> bool {
    if container_exists(config) {
        return true;
    }

    if !pull_image(config) {
        return false;
    }

    if !create_container(config) {
        eprintln!("  carrier: container creation failed");
        return false;
    }

    if !init_container(config) {
        eprintln!("  carrier: container initialization failed");
        let mut rm = run_as_user(config, "distrobox", &["rm", "--force", &config.container_name]);
        rm.status().ok();
        return false;
    }

    true
}

pub fn run_in_container(config: &Config, args: &[String]) -> i32 {
    let mut enter = run_as_user(config, "distrobox", &["enter", &config.container_name, "--", "sudo", "-E", &config.pm]);
    enter.args(args);

    let status = enter.status();
    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("carrier: failed to run command: {}", e);
            1
        }
    }
}

pub fn exec_in_container(config: &Config, cmd: &str, args: &[&str]) -> Result<String, String> {
    let mut enter = run_as_user(config, "distrobox", &["enter", "-n", &config.container_name, "--", cmd]);
    enter.args(args);

    let output = enter.output().map_err(|e| format!("exec failed: {}", e))?;
    if !output.status.success() {
        return Err(format!("{} exited with {}", cmd, output.status));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("invalid utf-8: {}", e))
}
