use app_core::structs::{HasOSLights, HasOSMech, System};
use std::io::{self, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor};
use tiny_http::{Header, Method, Response, Server};

fn main() {
  let system = Arc::new(Mutex::new(System::new(HasOSMech, HasOSLights)));
  let server = Server::http("127.0.0.1:3000").unwrap();

  {
    let mut sys = system.lock().unwrap();
    sys.internal_test = true;
    eprintln!("System created, internal_test = {}", sys.internal_test);
  }

  let mut stdout = io::stdout().into_raw_mode().unwrap();
  let html = std::fs::read_to_string("index.html").unwrap();
  let system_clone = system.clone();

  thread::spawn(move || {
    eprint!("Server running on http://127.0.0.1:3000");
    for mut request in server.incoming_requests() {
      let path = request.url();
      let method = request.method();

      match (method, path) {
        (Method::Get, "/") => {
          let response = Response::from_string(&html).with_header(
            Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
          );
          request.respond(response).ok();
        }
        (Method::Get, "/lights-status") => {
          let mut sys = system_clone.lock().unwrap();
          let lights = sys.get_light_status();
          let mut bits: u16 = 0;
          for (i, &light_on) in lights.iter().enumerate() {
            if light_on {
              bits |= 1 << i;
            }
          }
          request
            .respond(Response::from_data(bits.to_be_bytes()))
            .ok();
        }
        (Method::Post, "/toggle-button") => {
          let mut content = String::new();
          request.as_reader().read_to_string(&mut content).ok();
          let value: i32 = content.trim().parse().unwrap_or(-1);

          let mut sys = system_clone.lock().unwrap();
          match value {
            0 => {
              sys.auto_spa(None);
            }
            1 => {
              sys.toggle_jets();
            }
            2 => {
              sys.toggle_filter_schedule();
            }
            3 => {
              sys.toggle_quick_clean();
            }
            4 => {
              sys.toggle_main_valves();
            }
            6 => {
              sys.toggle_heater_on();
            }
            7 => {
              sys.toggle_heat_mode();
            }
            _ => {}
          }
          request.respond(Response::from_string("OK")).ok();
        }
        _ => {
          request
            .respond(Response::from_string("Not Found").with_status_code(404))
            .ok();
        }
      }
    }
  });

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

  let clear_all = |stdout: &mut dyn Write| {
    write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
    writeln!(stdout, "=== PoolMax System ===\r").unwrap();
    writeln!(stdout, "Server: http://127.0.0.1:3000\r").unwrap();
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

  let message_start_line = 14;
  let mut message_lines: Vec<String> = Vec::new();
  let max_messages = 48;

  loop {
    if let Ok(key) = rx.try_recv() {
      use termion::event::Key;

      let mut sys = system.lock().unwrap();

      match key {
        Key::Char('c') | Key::Char('C') => {
          sys.toggle_quick_clean();
        }
        Key::Char('r') | Key::Char('R') => {
          sys.toggle_filter_schedule();
        }
        Key::Char('h') | Key::Char('H') => {
          sys.toggle_heater_on();
        }
        Key::Char('j') | Key::Char('J') => {
          sys.toggle_jets();
        }
        Key::Char('k') | Key::Char('K') => {
          sys.toggle_heat_mode();
        }
        Key::Char('s') | Key::Char('S') => {
          sys.auto_spa(None);
        }
        Key::Char('p') | Key::Char('P') => {
          sys.display_status();
        }
        Key::Char('m') | Key::Char('M') => {
          sys.toggle_main_valves();
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

    {
      let mut sys = system.lock().unwrap();
      while let Some(msg) = sys.pop_message() {
        message_lines.push(msg.to_string());
        has_new_messages = true;

        if message_lines.len() > max_messages {
          message_lines.remove(0);
        }
      }
    }

    if has_new_messages {
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
