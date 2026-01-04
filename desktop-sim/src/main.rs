use app_core::structs::{HasOSLights, HasOSMech, System};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::io::{self, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

async fn handle_request(
  req: Request<hyper::body::Incoming>,
  system: Arc<Mutex<System<HasOSMech, HasOSLights>>>,
  html: Arc<String>,
) -> Result<Response<Full<Bytes>>, Infallible> {
  let path = req.uri().path();
  let method = req.method();

  match (method.as_str(), path) {
    ("GET", "/") => {
      let mut res = Response::new(Full::new(Bytes::from(html.as_ref().clone())));
      res.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        "text/html; charset=utf-8".parse().unwrap(),
      );
      Ok(res)
    }
    ("GET", "/lights-status") => {
      let mut sys = system.lock().unwrap();

      let lights = sys.get_light_status();
      let mut bits: u16 = 0;
      for (i, &light_on) in lights.iter().enumerate() {
        if light_on {
          bits |= 1 << i;
        }
      }

      let bytes = bits.to_be_bytes();

      Ok(Response::new(Full::new(Bytes::from(bytes.to_vec()))))
    }
    ("GET", "/status") => {
      let sys = system.lock().unwrap();
      let status = format!("System status: internal_test={}", sys.internal_test);
      Ok(Response::new(Full::new(Bytes::from(status))))
    }
    ("POST", "/toggle-button") => {
      let body = req.into_body(); // Extract body from request
      let body_bytes = body.collect().await.unwrap().to_bytes();
      let body_str = std::str::from_utf8(&body_bytes).unwrap();
      let value: i32 = body_str.trim().parse().unwrap();
      let mut sys = system.lock().unwrap();

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
        _ => println!("error"),
      }

      Ok(Response::new(Full::new(Bytes::from("Quick clean toggled"))))
    }
    _ => {
      let mut response = Response::new(Full::new(Bytes::from("Not Found")));
      *response.status_mut() = StatusCode::NOT_FOUND;
      Ok(response)
    }
  }
}

async fn run_server(
  system: Arc<Mutex<System<HasOSMech, HasOSLights>>>,
  html: Arc<String>,
) -> Result<(), Box<dyn std::error::Error>> {
  let addr = "127.0.0.1:3000";
  let listener = TcpListener::bind(addr).await?;

  eprintln!("Server running on http://{}", addr);

  loop {
    let (stream, _) = listener.accept().await?;
    let io = TokioIo::new(stream);
    let system = system.clone();
    let html = html.clone();

    tokio::task::spawn(async move {
      if let Err(err) = http1::Builder::new()
        .serve_connection(
          io,
          service_fn(move |req| handle_request(req, system.clone(), html.clone())),
        )
        .await
      {
        eprintln!("Error serving connection: {:?}", err);
      }
    });
  }
}

fn main() {
  let system = Arc::new(Mutex::new(System::new(HasOSMech, HasOSLights)));
  {
    let mut sys = system.lock().unwrap();
    sys.internal_test = true;
    eprintln!("System created, internal_test = {}", sys.internal_test);
  }

  let mut stdout = io::stdout().into_raw_mode().unwrap();

  let html = Arc::new(std::fs::read_to_string("index.html").expect("Failed to read index.html"));

  let system_server = system.clone();
  let html_server = html.clone();

  thread::spawn(move || {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
      if let Err(e) = run_server(system_server, html_server).await {
        eprintln!("Server error: {}", e);
      }
    });
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
