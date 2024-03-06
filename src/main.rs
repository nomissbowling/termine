#![doc(html_root_url = "https://docs.rs/termine/0.1.1")]
//! termine mine for Rust with termion
//!

use std::error::Error;
use std::time;
use std::sync::mpsc;

use termion::event::{Event, Key, MouseEvent};
use termion::{color, color::Rgb};

use termioff::Termioff;

/// Termine
pub struct Termine {
  /// area width
  pub w: u16,
  /// area height
  pub h: u16,
  /// mines
  pub m: u16,
  /// field w x h
  pub f: Vec<Vec<u8>>,
  /// cursor row
  pub r: u16,
  /// cursor column
  pub c: u16,
  /// ms timeout for idle
  pub ms: time::Duration,
  /// blink cursor count max
  pub b: u16,
  /// tick count about b x ms
  pub t: u16
}

/// Termine
impl Termine {
  /// constructor
  pub fn new(w: u16, h: u16, m: u16) -> Self {
    let f = (0..h).into_iter().map(|_r|
      (0..w).into_iter().map(|_c| 0).collect()).collect();
    Termine{w, h, m, f, r: 0, c: 0,
      ms: time::Duration::from_millis(10), b: 80, t: 0}
  }

  /// refresh
  pub fn refresh(&self, tm: &mut Termioff) -> Result<(), Box<dyn Error>> {
    for (r, v) in (&self.f).iter().enumerate() {
      for (c, _u) in v.iter().enumerate() {
        let cbf = self.c(r as u16, c as u16)?;
        tm.wr(5 + c as u16, 5 + r as u16, 3, cbf.1, cbf.2, &cbf.0)?;
      }
    }
    Ok(())
  }

  /// c
  /// upper 4bit
  /// - 7 skip
  /// - 6 1: flag, 0: as is
  /// - 5 1: question, 0: as is
  /// - 4 1: open, 0: close
  /// lower 4bit
  /// - 0-3 0: '_', 1-8: num, 9-14: skip, 15: '@' mine
  pub fn c(&self, r: u16, c: u16) ->
    Result<(String, Rgb, Rgb), Box<dyn Error>> {
    let f = "LO??PPPP++++++++".chars().collect::<Vec<_>>(); // 4 bit upper
    let s = "_12345678......@".chars().collect::<Vec<_>>(); // 4 bit lower
    let curs = r == self.r && c == self.c;
    let n = if curs { s[0] } else { f[0] };
    let o = if self.t < self.b / 2 && curs { f[15] } else { n };
    Ok((String::from_utf8(vec![o as u8])?, Rgb(32, 32, 32), Rgb(240, 192, 32)))
  }

  /// tick and control blink cursor
  pub fn tick(&mut self, tm: &mut Termioff) -> Result<(), Box<dyn Error>> {
    self.t += 1;
    if self.t == self.b / 2 { self.refresh(tm)?; }
    else if self.t >= self.b { self.reset_t(tm)?; }
    Ok(())
  }

  /// reset tick
  pub fn reset_t(&mut self, tm: &mut Termioff) -> Result<(), Box<dyn Error>> {
    self.t = 0;
    self.refresh(tm)?;
    Ok(())
  }

  /// update
  pub fn update(&mut self, k: Key) -> Result<bool, Box<dyn Error>> {
    let mut f = true;
    match k {
    Key::Left | Key::Char('h') => { if self.c > 0 { self.c -= 1; } },
    Key::Down | Key::Char('j') => { if self.r < self.h - 1 { self.r += 1; } },
    Key::Up | Key::Char('k') => { if self.r > 0 { self.r -= 1; } },
    Key::Right | Key::Char('l') => { if self.c < self.w - 1 { self.c += 1; } },
    _ => { f = false; }
    }
    Ok(f)
  }
}

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

  let mut m = Termine::new(8, 8, 5);
  m.reset_t(&mut tm)?;

  let (_tx, rx) = tm.prepare_thread()?;
  loop {
    // thread::sleep(ms);
    match rx.recv_timeout(m.ms) {
    Err(mpsc::RecvTimeoutError::Disconnected) => break, // not be arrived here
    Err(mpsc::RecvTimeoutError::Timeout) => { // idle
      tm.wr(40, 10, 1, Rgb(192, 192, 192), Rgb(8, 8, 8), &msg(m.b, m.t, t))?;
      m.tick(&mut tm)?;
    },
    Ok(ev) => {
      match ev {
      Ok(Event::Key(k)) => {
        match k {
        Key::Ctrl('c') | Key::Char('q') => break,
        Key::Esc | Key::Char('\x1b') => break,
        _ => { if m.update(k)? { m.reset_t(&mut tm)?; } }
        }
      },
      Ok(Event::Mouse(m)) => {
        match m {
        MouseEvent::Press(_btn, x, y) => {
          tm.wr(2, 48, 1, color::Cyan, color::Green, &msg(x, y, t))?;
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
