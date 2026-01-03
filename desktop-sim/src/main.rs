use app_core::structs::{HasOSLights, HasOSMech, System};
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor};

fn main() {
  let mut system = System::new(HasOSMech, HasOSLights);
  system.internal_test = true;
  let mut stdout = io::stdout().into_raw_mode().unwrap();

  eprintln!("System created, internal_test = {}", system.internal_test);

  // Channel to send key presses from thread to main loop
  let (tx, rx) = mpsc::channel();

  // Thread to capture keyboard input
  thread::spawn(move || {
    let stdin = io::stdin();
    for key in stdin.keys() {
      if let Ok(k) = key {
        tx.send(k).ok();
      }
    }
  });

  // Define as a function that takes stdout as a parameter
  let clear_all = |stdout: &mut dyn Write| {
    write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
    writeln!(stdout, "=== PoolMax System ===\r").unwrap();
    writeln!(stdout, "Controls:\r").unwrap();
    writeln!(stdout, "  c - Toggle Quick Clean\r").unwrap();
    writeln!(stdout, "  r - Toggle Filter Schedule\r").unwrap();
    writeln!(stdout, "  h - Heater On\r").unwrap();
    writeln!(stdout, "  j - Toggle Jets\r").unwrap();
    writeln!(stdout, "  s - Spa Mode\r").unwrap();
    writeln!(stdout, "  m - Switch Main Valve Orientation (Pool/Spa)\r").unwrap();
    writeln!(stdout, "  k - Switch Heater Mode\r").unwrap();
    writeln!(stdout, "  l - Clear Screen\r").unwrap();
    writeln!(stdout, "  q - Quit\r").unwrap();
    writeln!(stdout, "\r").unwrap();
    writeln!(stdout, "=== Messages ===\r").unwrap();
    stdout.flush().unwrap();
  };

  clear_all(&mut stdout);

  let message_start_line = 13;
  let mut message_lines: Vec<String> = Vec::new();
  let max_messages = 48;

  loop {
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
          system.toggle_jets();
        }
        Key::Char('k') | Key::Char('K') => {
          system.toggle_heat_mode();
        }
        Key::Char('s') | Key::Char('S') => {
          system.auto_spa(None);
        }
        Key::Char('p') | Key::Char('P') => {
          system.display_status();
        }
        Key::Char('m') | Key::Char('M') => {
          system.toggle_main_valves();
        }
        Key::Char('l') | Key::Char('L') => {
          message_lines.clear();
          clear_all(&mut stdout);
        }
        Key::Char('q') | Key::Char('Q') => {
          write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
          writeln!(stdout, "Quitting...\r").unwrap();
          break;
        }
        _ => {}
      }
    }

    let mut has_new_messages = false;

    while let Some(msg) = system.pop_message() {
      message_lines.push(msg.to_string());
      has_new_messages = true;

      if message_lines.len() > max_messages {
        message_lines.remove(0);
      }
    }

    if has_new_messages {
      // Redraw message area
      message_lines.push("--------------------------------------".to_string());

      for (i, line) in message_lines.iter().enumerate() {
        write!(
          stdout,
          "{}{}{}",
          cursor::Goto(1, message_start_line + i as u16),
          clear::CurrentLine,
          line
        )
        .unwrap();
        writeln!(stdout, "\r").unwrap();
      }
    }

    stdout.flush().unwrap();

    thread::sleep(Duration::from_millis(50));
  }
}
// // For embedded/microcontroller implementation with LED screen
// #[cfg(not(target_os = "linux"))]
// fn display_on_led(system: &mut System<impl Mech, impl Lights>) {
//   // This would be called in your embedded main loop
//   if let Some(msg) = system.pop_message() {
//     let msg_str = msg.get_str();

//     // Display on LED screen (pseudo-code)
//     // led_display.clear();
//     // led_display.write_str(msg_str);
//     // led_display.refresh();
//   }
// }
