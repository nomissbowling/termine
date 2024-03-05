#![doc(html_root_url = "https://docs.rs/termine/0.1.0")]
//! termine mine for Rust with termion
//!

use std::error::Error;
use std::time;
use std::sync::mpsc;

use termion::event::{Event, Key, MouseEvent};
use termion::{color, color::Rgb};

use termioff::Termioff;

/// msg
pub fn msg(x: u16, y: u16, t: time::Instant) -> String {
  format!("[({}, {}) {:?}]", x, y, t.elapsed())
}

/// main
pub fn main() -> Result<(), Box<dyn Error>> {
  let mut tm = Termioff::new(2)?;
  tm.begin()?;

  let t = time::Instant::now();
  tm.wr(1, 1, 3, color::Magenta, Rgb(240, 192, 32), &msg(tm.w, tm.h, t))?;

  let mut r = 10u16;
  let mut c = 40u16;

  let (_tx, rx) = tm.prepare_thread()?;
  let ms = time::Duration::from_millis(10); // timeout for idle
  loop {
    // thread::sleep(ms);
    match rx.recv_timeout(ms) {
    Err(mpsc::RecvTimeoutError::Disconnected) => break, // not be arrived here
    Err(mpsc::RecvTimeoutError::Timeout) => { // idle
      tm.wr(c, r, 1, Rgb(192, 192, 192), Rgb(8, 8, 8), &msg(c, r, t))?;
    },
    Ok(ev) => {
      match ev {
      Ok(Event::Key(k)) => {
        match k {
        Key::Ctrl('c') | Key::Char('q') => break,
        Key::Esc | Key::Char('\x1b') => break,
        Key::Left | Key::Char('h') => { if c > 1 { c -= 1; } },
        Key::Down | Key::Char('j') => { if r < tm.h { r += 1; } },
        Key::Up | Key::Char('k') => { if r > 1 { r -= 1; } },
        Key::Right | Key::Char('l') => { if c < tm.w { c += 1; } },
        _ => ()
        }
      },
      Ok(Event::Mouse(m)) => {
        match m {
        MouseEvent::Press(_btn, x, y) => {
          tm.wr(30, 5, 1, color::Cyan, color::Green, &msg(x, y, t))?;
        },
        _ => ()
        }
      },
      _ => ()
      }
    }
    }
  }

  // handle.join()?;

  tm.wr(1, tm.h - 1, 3, Rgb(255, 0, 0), Rgb(255, 255, 0), &msg(tm.w, tm.h, t))?;
  tm.fin()?;
  Ok(())
}

/// test with [-- --nocapture] or [-- --show-output]
#[cfg(test)]
mod tests {
  // use super::*;

  /// test a
  #[test]
  fn test_a() {
    assert_eq!(true, true);
  }
}
