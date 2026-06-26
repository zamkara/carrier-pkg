use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const BRAILLE: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

pub fn spin_run(msg: &str, cmd: &mut Command) -> bool {
    let done = Arc::new(AtomicBool::new(false));
    let d = Arc::clone(&done);
    let msg_owned = msg.to_string();

    print!("{}  ", msg);
    std::io::stdout().flush().ok();

    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let stderr = child.stderr.take();

    let msg_clone = msg_owned.clone();
    let handle = thread::spawn(move || {
        let mut i = 0;
        while !d.load(Ordering::Relaxed) {
            print!("\r{} {}", msg_clone, BRAILLE[i]);
            std::io::stdout().flush().ok();
            thread::sleep(Duration::from_millis(100));
            i = (i + 1) % BRAILLE.len();
        }
    });

    let status = child.wait();
    done.store(true, Ordering::Relaxed);
    handle.join().ok();

    print!("\r{}", " ".repeat(msg_owned.len() + 3));
    std::io::stdout().flush().ok();

    match status {
        Ok(s) if s.success() => {
            println!("\r\u{2714} {} done", msg);
            true
        }
        _ => {
            println!("\r\u{2717} {} failed", msg);
            if let Some(mut err) = stderr {
                let mut buf = String::new();
                err.read_to_string(&mut buf).ok();
                if !buf.is_empty() {
                    print!("{}", buf);
                }
            }
            false
        }
    }
}
