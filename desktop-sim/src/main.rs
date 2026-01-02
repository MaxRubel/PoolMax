use app_core::{HasOS, System};
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
fn main() {
    let mut system = System::new(HasOS);
    system.internal_test = true;
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
    writeln!(stdout, "  c - Toggle Quick Clean\r").unwrap();
    writeln!(stdout, "  r - Toggle Filter Schedule\r").unwrap();
    writeln!(stdout, "  h - Heater On\r").unwrap();
    writeln!(stdout, "  j - Heater Off\r").unwrap();
    writeln!(stdout, "  s - Spa Mode\r").unwrap();
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
                Key::Char('c') | Key::Char('C') => {
                    system.toggle_quick_clean();
                }
                Key::Char('r') | Key::Char('R') => {
                    system.toggle_filter_schedule();
                }
                Key::Char('h') | Key::Char('H') => {
                    system.toggle_heater_on();
                }
                Key::Char('j') | Key::Char('J') => {
                    system.set_heater_on(false);
                }
                Key::Char('n') | Key::Char('M') => {
                    system.toggle_jets();
                }
                Key::Char('s') | Key::Char('S') => {
                    system.auto_spa();
                }

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
