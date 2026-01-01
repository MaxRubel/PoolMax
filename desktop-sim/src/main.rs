use app_core::{HasOS, System};
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
fn main() {
    let mut system = System::new(HasOS);
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    // Channel to send key presses from thread to main loop
    let (tx, rx) = mpsc::channel();

    // Thread to capture keyboard input (simulates ISR)
    thread::spawn(move || {
        let stdin = io::stdin();
        for key in stdin.keys() {
            if let Ok(k) = key {
                tx.send(k).ok();
            }
        }
    });

    writeln!(stdout, "Controls:\r").unwrap();
    writeln!(stdout, "  f - Toggle Filter\r").unwrap();
    writeln!(stdout, "  h - Toggle Heater\r").unwrap();
    writeln!(stdout, "  j - Toggle Jets\r").unwrap();
    writeln!(stdout, "  m - Switch System Mode (Pool/Spa)\r").unwrap();
    writeln!(stdout, "  v - Pool: Vacuum mode\r").unwrap();
    writeln!(stdout, "  s - Pool: Skimmer mode\r").unwrap();
    writeln!(stdout, "  b - Pool: Blend mode\r").unwrap();
    writeln!(stdout, "  q - Quit\r").unwrap();
    writeln!(stdout, "\r").unwrap();
    stdout.flush().unwrap();

    loop {
        // Check for key presses (simulates checking ISR flag)
        if let Ok(key) = rx.try_recv() {
            use termion::event::Key;

            match key {
                Key::Char('f') | Key::Char('F') => {
                    if system.filter_on {
                        system.stop_filter();
                        writeln!(stdout, "Filter OFF\r").unwrap();
                    } else {
                        system.start_filter();
                        writeln!(stdout, "Filter ON\r").unwrap();
                    }
                }
                Key::Char('h') | Key::Char('H') => {
                    if system.heater_on {
                        system.stop_heater();
                        writeln!(stdout, "Heater OFF\r").unwrap();
                    } else {
                        system.start_heater();
                        writeln!(stdout, "Heater ON\r").unwrap();
                    }
                }
                Key::Char('j') | Key::Char('J') => {
                    if system.jets_on {
                        system.stop_jets();
                        writeln!(stdout, "Jets OFF\r").unwrap();
                    } else {
                        system.start_jets();
                        writeln!(stdout, "Jets ON\r").unwrap();
                    }
                }
                // Key::Char('v') | Key::Char('V') => {
                //     system.change_pool_mode(app_core::PoolMode::Vacuum);
                //     writeln!(stdout, "Pool Mode: Vacuum\r").unwrap();
                // }
                // Key::Char('s') | Key::Char('S') => {
                //     system.change_pool_mode(app_core::PoolMode::Skimmer);
                //     writeln!(stdout, "Pool Mode: Skimmer\r").unwrap();
                // }
                // Key::Char('b') | Key::Char('B') => {
                //     system.change_pool_mode(app_core::PoolMode::Blend);
                //     writeln!(stdout, "Pool Mode: Blend\r").unwrap();
                // }
                Key::Char('q') | Key::Char('Q') => {
                    writeln!(stdout, "Quitting...\r").unwrap();
                    break;
                }
                _ => {}
            }
            stdout.flush().unwrap();
        }

        // Main loop tick (simulates embedded main loop)
        thread::sleep(Duration::from_millis(50));
    }
}
