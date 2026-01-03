// message_queue.rs - Add this as a new module
use core::fmt::Write as FmtWrite;

/// Fixed-size circular buffer for log messages
pub struct MessageQueue<const N: usize> {
  messages: [Option<Message>; N],
  head: usize,
  tail: usize,
  count: usize,
}

#[derive(Clone, Copy)]
pub struct Message {
  buffer: [u8; 128],
  len: usize,
}

impl Message {
  pub fn new() -> Self {
    Message {
      buffer: [0; 128],
      len: 0,
    }
  }

  pub fn get_str(&self) -> &str {
    core::str::from_utf8(&self.buffer[..self.len]).unwrap_or("")
  }
}

impl FmtWrite for Message {
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    let bytes = s.as_bytes();
    let available = self.buffer.len() - self.len;
    let to_copy = bytes.len().min(available);

    self.buffer[self.len..self.len + to_copy].copy_from_slice(&bytes[..to_copy]);
    self.len += to_copy;

    Ok(())
  }
}

impl<const N: usize> MessageQueue<N> {
  pub const fn new() -> Self {
    MessageQueue {
      messages: [None; N],
      head: 0,
      tail: 0,
      count: 0,
    }
  }

  pub fn push(&mut self, msg: Message) {
    self.messages[self.head] = Some(msg);
    self.head = (self.head + 1) % N;

    if self.count == N {
      // Queue is full, move tail forward (overwrite oldest)
      self.tail = (self.tail + 1) % N;
    } else {
      self.count += 1;
    }
  }

  pub fn pop(&mut self) -> Option<Message> {
    if self.count == 0 {
      return None;
    }

    let msg = self.messages[self.tail].take();
    self.tail = (self.tail + 1) % N;
    self.count -= 1;

    msg
  }

  pub fn peek(&self) -> Option<&Message> {
    if self.count == 0 {
      None
    } else {
      self.messages[self.tail].as_ref()
    }
  }

  pub fn len(&self) -> usize {
    self.count
  }

  pub fn is_empty(&self) -> bool {
    self.count == 0
  }

  pub fn is_full(&self) -> bool {
    self.count == N
  }

  pub fn clear(&mut self) {
    self.messages = [None; N];
    self.head = 0;
    self.tail = 0;
    self.count = 0;
  }
}

#[macro_export]
macro_rules! log_msg {
    ($queue:expr, $($arg:tt)*) => {{
        let mut msg = $crate::message_queue::Message::new();
        use core::fmt::Write;
        let _ = write!(msg, $($arg)*);
        $queue.push(msg);
    }};
}

#[cfg(test)]
mod tests {
  use super::*;
  use core::fmt::Write;

  #[test]
  fn test_queue_basic() {
    let mut queue = MessageQueue::<4>::new();

    let mut msg = Message::new();
    write!(msg, "Test message").unwrap();
    queue.push(msg);

    assert_eq!(queue.len(), 1);
    assert!(!queue.is_empty());

    let popped = queue.pop().unwrap();
    assert_eq!(popped.get_str(), "Test message");
    assert!(queue.is_empty());
  }

  #[test]
  fn test_queue_overflow() {
    let mut queue = MessageQueue::<3>::new();

    for i in 0..5 {
      let mut msg = Message::new();
      write!(msg, "Message {}", i).unwrap();
      queue.push(msg);
    }

    // Should only have last 3 messages
    assert_eq!(queue.len(), 3);
    assert_eq!(queue.pop().unwrap().get_str(), "Message 2");
    assert_eq!(queue.pop().unwrap().get_str(), "Message 3");
    assert_eq!(queue.pop().unwrap().get_str(), "Message 4");
  }
}
