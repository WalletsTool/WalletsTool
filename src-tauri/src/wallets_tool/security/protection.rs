use std::thread;
use std::time::Duration;

#[cfg(windows)]
use winapi::um::debugapi::IsDebuggerPresent;

pub fn enable_protection() {
    #[cfg(windows)]
    thread::spawn(|| {
        loop {
            unsafe {
                if IsDebuggerPresent() != 0 {
                    // Debugger detected!
                    // In a real production app, we should log this (securely) or just exit.
                    std::process::exit(1);
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}
