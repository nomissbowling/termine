#![doc(html_root_url = "https://docs.rs/termine/3.1.0")]
//! termine mine for Rust with termion
//!

use std::error::Error;
use std::time;
use std::sync::mpsc;

use termion::event::{Event, Key, MouseEvent};
use termion::{color, color::Rgb};

use termioff::Termioff;

use minefield::MineField;

use mvc_rs::View as MVCView;

/// Term
pub struct Term<T> {
  /// colors
  pub colors: Vec<T>,
  /// Termioff
  pub tm: Termioff
}

/// trait MVCView for Term
impl<T: color::Color + Clone> MVCView<T> for Term<T> {
  /// wr
  fn wr(&mut self, x: u16, y: u16, st: u16,
    bgc: u16, fgc: u16, msg: &String) -> Result<(), Box<dyn Error>> {
    self.tm.wr(x + 1, y + 1, st, self.col(bgc), self.col(fgc), msg)?;
    Ok(())
  }

  /// reg
  fn reg(&mut self, c: Vec<T>) -> () {
    self.colors = c;
  }

  /// col
  fn col(&self, n: u16) -> T {
    self.colors[n as usize].clone()
  }
}

/// Term
impl<T: color::Color + Clone> Term<T> {
  /// constructor
  pub fn new(k: u16) -> Result<Self, Box<dyn Error>> {
    Ok(Term{colors: vec![], tm: Termioff::new(k)?})
  }
}

/// Termine
pub struct Termine {
  /// minefield
  pub m: MineField,
  /// view
  pub v: Term<Rgb>,
  /// time Instant
  pub t: time::Instant
}

/// Drop for Termine
impl Drop for Termine {
  /// destructor
  fn drop(&mut self) {
    self.v.tm.fin().expect("fin");
  }
}

/// Termine
impl Termine {
  /// constructor
  pub fn new(m: MineField) -> Result<Self, Box<dyn Error>> {
    let mut s = Termine{m, v: Term::new(2)?, t: time::Instant::now()};
    let colors = [ // bgc fgc
      [96, 240, 32, 0], [32, 96, 240, 0], // closed
      [32, 96, 240, 0], [240, 192, 32, 0], // opened
      [240, 32, 96, 0], [240, 192, 32, 0] // ending
    ].into_iter().map(|c| Rgb(c[0], c[1], c[2])).collect::<Vec<_>>();
    s.v.reg(colors);
    s.v.tm.begin()?;
    s.m.reset_tick(&mut s.v)?;
    Ok(s)
  }

  /// status terminal
  pub fn status_t(&mut self, h: u16, st: u16,
    bgc: impl color::Color, fgc: impl color::Color) ->
    Result<(), Box<dyn Error>> {
    self.v.tm.wr(1, self.v.tm.h - h + 1, st, bgc, fgc,
      &self.msg(self.v.tm.w, self.v.tm.h))?;
    Ok(())
  }

  /// status mouse
  pub fn status_p(&mut self, h: u16, st: u16,
    bgc: impl color::Color, fgc: impl color::Color, x: u16, y: u16) ->
    Result<(), Box<dyn Error>> {
    self.v.tm.wr(1, self.v.tm.h - h + 1, st, bgc, fgc,
      &self.msg(x, y))?;
    Ok(())
  }

  /// status minefield
  pub fn status_m(&mut self, h: u16, st: u16,
    bgc: impl color::Color, fgc: impl color::Color) ->
    Result<(), Box<dyn Error>> {
    self.v.tm.wr(1, self.v.tm.h - h + 1, st, bgc, fgc,
      &self.msg(self.m.m, self.m.s & 0x3fff))?;
    Ok(())
  }

  /// msg
  pub fn msg(&self, x: u16, y: u16) -> String {
    format!("({}, {}) {:?}", x, y, self.t.elapsed())
  }

  /// key
  pub fn key(&mut self, k: Key) -> bool {
    let mut f = true;
    match k {
    Key::Left | Key::Char('h') => { self.m.left(); },
    Key::Down | Key::Char('j') => { self.m.down(); },
    Key::Up | Key::Char('k') => { self.m.up(); },
    Key::Right | Key::Char('l') => { self.m.right(); },
    Key::Char(' ') => { self.m.click(); },
    _ => { f = false; }
    }
    f
  }

  /// proc
  pub fn proc(&mut self, rx: &mpsc::Receiver<Result<Event, std::io::Error>>) ->
    Result<bool, Box<dyn Error>> {
    // thread::sleep(self.m.ms);
    match rx.recv_timeout(self.m.ms) {
    Err(mpsc::RecvTimeoutError::Disconnected) => Err("Disconnected".into()),
    Err(mpsc::RecvTimeoutError::Timeout) => { // idle
      self.status_m(3, 1, Rgb(192, 192, 192), Rgb(8, 8, 8))?;
      self.m.tick(&mut self.v)?;
      Ok(true)
    },
    Ok(ev) => {
      Ok(match ev {
      Ok(Event::Key(k)) => {
        let f = match k {
        Key::Ctrl('c') | Key::Char('q') => false,
        Key::Esc | Key::Char('\x1b') => false,
        _ => true
        };
        if !f { return Ok(false); }
        if self.key(k) { self.m.reset_tick(&mut self.v)?; }
        if self.m.is_end() { self.m.ending(&mut self.v)?; return Ok(false); }
        true
      },
      Ok(Event::Mouse(m)) => {
        match m {
/*
        MouseEvent::Move(x, y) => {
          self.status_p(5, 1, color::Blue, color::Yellow, x, y)?;
          if self.m.update_m(x, y) { self.m.reset_tick(&mut self.v)?; }
          true
        },
*/
        MouseEvent::Press(_btn, x, y) => {
          self.status_p(4, 1, color::Cyan, color::Green, x, y)?;
          if self.m.click() { self.m.reset_tick(&mut self.v)?; }
          if self.m.is_end() { self.m.ending(&mut self.v)?; return Ok(false); }
          true
        },
        _ => true
        }
      },
/*
      Ok(Event::Resize(_w, _h)) => {
        true
      },
*/
      _ => true
      })
    }
    }
  }

  /// mainloop
  pub fn mainloop(&mut self) -> Result<(), Box<dyn Error>> {
    let (_tx, rx) = self.v.tm.prepare_thread()?;
    loop { if !self.proc(&rx)? { break; } }
    // handle.join()?;
    Ok(())
  }
}

/// main
pub fn main() -> Result<(), Box<dyn Error>> {
  // let m = MineField::new(1, 1, 0);
  // let m = MineField::new(1, 1, 1);
  // let m = MineField::new(2, 2, 0);
  // let m = MineField::new(2, 2, 2);
  // let m = MineField::new(5, 4, 3);
  // let m = MineField::new(8, 8, 5);
  let m = MineField::new(16, 8, 12);
  // let m = MineField::new(80, 50, 12);
  let mut g = Termine::new(m)?;
  g.status_t(1, 3, color::Magenta, Rgb(240, 192, 32))?;
  g.mainloop()?;
  g.status_m(3, 1, Rgb(240, 192, 32), Rgb(192, 32, 240))?;
  g.status_t(2, 3, Rgb(255, 0, 0), Rgb(255, 255, 0))?;
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
