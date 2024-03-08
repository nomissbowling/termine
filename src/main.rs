#![doc(html_root_url = "https://docs.rs/termine/1.1.0")]
//! termine mine for Rust with termion
//!

use std::error::Error;
use std::time;
use std::sync::mpsc;

use rand;
use rand::prelude::SliceRandom;

use termion::event::{Event, Key, MouseEvent};
use termion::{color, color::Rgb};

use termioff::Termioff;

/// Termine
pub struct Termine {
  /// status
  pub s: u16,
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
      (0..w).into_iter().map(|_c|
        0).collect()).collect(); // all close
    Termine{s: 0, w, h, m, f, r: 0, c: 0,
      ms: time::Duration::from_millis(10), b: 80, t: 0}
  }

  /// refresh
  pub fn refresh(&self, tm: &mut Termioff) -> Result<(), Box<dyn Error>> {
    for (r, v) in self.f.iter().enumerate() {
      for (c, u) in v.iter().enumerate() {
        let ur = r as u16;
        let uc = c as u16;
        let (s, bgc, fgc) = self.c(ur, uc, *u)?;
        tm.wr(1 + uc, 1 + ur, 3, bgc, fgc, &s)?;
      }
    }
    Ok(())
  }

  /// c
  /// upper 4bit
  /// - 7 1: force open at ending, 0: normal
  /// - 6 1: flag, 0: as is
  /// - 5 1: question, 0: as is
  /// - 4 1: open, 0: close
  /// lower 4bit
  /// - 0-3 0: '_', 1-8: num, 9-14: skip, 15: '@' mine
  pub fn c(&self, r: u16, c: u16, u: u8) ->
    Result<(String, Rgb, Rgb), Box<dyn Error>> {
    let f = "L*??PPPP++++++++".chars().collect::<Vec<_>>(); // 4 bit upper
    let s = "_12345678......@".chars().collect::<Vec<_>>(); // 4 bit lower
    let v = Self::get_v(u);
    let n = if self.is_opened(r, c) { s[v as usize] } else { f[0] };
    let curs = r == self.r && c == self.c;
    let o = if !curs || self.is_success() { n } else { // through
      if self.is_explosion() && Self::is_mine(v) { f[1] } // may be always mine
      else { if self.is_blink() { f[15] } else { n } } // blink or through
    };
    let (bgc, fgc) = if Self::is_e(u) { (Rgb(240, 32, 96), Rgb(240, 192, 32)) }
      else if Self::is_o(u) { (Rgb(32, 96, 240), Rgb(240, 192, 32)) }
      else { (Rgb(96, 240, 32), Rgb(32, 96, 240)) };
    Ok((String::from_utf8(vec![o as u8])?, bgc, fgc))
  }

  /// is_blink
  pub fn is_blink(&self) -> bool { self.t < self.b / 2 }

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
  pub fn update(&mut self, k: Key) -> bool {
    let mut f = true;
    match k {
    Key::Left | Key::Char('h') => { if self.c > 0 { self.c -= 1; } },
    Key::Down | Key::Char('j') => { if self.r < self.h - 1 { self.r += 1; } },
    Key::Up | Key::Char('k') => { if self.r > 0 { self.r -= 1; } },
    Key::Right | Key::Char('l') => { if self.c < self.w - 1 { self.c += 1; } },
    Key::Char(' ') => {
      if self.s == 0 { self.start(); } // at the first time
      if !self.is_opened(self.r, self.c) {
        if !self.open(self.r, self.c) { self.explosion(); }
        else {
          if self.s + self.m == self.w*self.h { self.success(); } // not '>='
        }
      }
    },
    _ => { f = false; }
    }
    f
  }

  /// is_opened
  pub fn is_opened(&self, r: u16, c: u16) -> bool {
    Self::is_o(self.f[r as usize][c as usize])
  }

  /// open
  pub fn open(&mut self, r: u16, c: u16) -> bool {
    let n = &mut self.f[r as usize][c as usize];
    let v = Self::get_v(*n);
    if Self::is_mine(v) { return false; } // explosion
    Self::set_o(n, false);
    self.s += 1;
    if v == 0 {
      let rs = if r > 0 { r - 1 } else { r };
      let re = if r < self.h - 1 { r + 1 } else { r };
      let cs = if c > 0 { c - 1 } else { c };
      let ce = if c < self.w - 1 { c + 1 } else { c };
      for j in rs..=re {
        for i in cs..=ce {
          if j == r && i == c { continue; }
          if !self.is_opened(j, i) { self.open(j, i); } // always success
        }
      }
    }
    true
  }

  /// is_explosion
  pub fn is_explosion(&self) -> bool { self.s & 0x8000 != 0 }

  /// explosion
  pub fn explosion(&mut self) -> () { self.s |= 0x8000; }

  /// is_success
  pub fn is_success(&self) -> bool { self.s & 0x4000 != 0 }

  /// success
  pub fn success(&mut self) -> () { self.s |= 0x4000; }

  /// is_end
  pub fn is_end(&self) -> bool { self.s >= 0x4000 }

  /// ending
  pub fn ending(&mut self, tm: &mut Termioff) -> Result<(), Box<dyn Error>> {
    for v in &mut self.f { for u in v { Self::set_o(u, true); } } // all open
    self.refresh(tm)?;
    Ok(())
  }

  /// start
  pub fn start(&mut self) -> () {
    let e = self.m >= self.w*self.h; // fill all when mine full
    let mut p: Vec<u16> = (0..self.w*self.h).into_iter().collect();
    p.shuffle(&mut rand::thread_rng());
    let mut n = 0;
    for i in 0..=self.m as usize {
      if n >= self.m || i >= p.len() { break; }
      let r = p[i] / self.w;
      let c = p[i] % self.w;
      if e || r != self.r || c != self.c { // fill all when mine full
        Self::set_m(&mut self.f[r as usize][c as usize]);
        n += 1;
      }
    }
    let f = self.f.clone();
    for (r, v) in self.f.iter_mut().enumerate() {
      for (c, u) in v.iter_mut().enumerate() {
        if Self::is_mine(f[r][c]) { continue; }
        *u = Self::get_k(self.w, self.h, &f, r as u16, c as u16);
      }
    }
    ()
  }

  /// get_k
  pub fn get_k(w: u16, h: u16, f: &Vec<Vec<u8>>, r: u16, c: u16) -> u8 {
    let mut n = 0u8;
    let rs = if r > 0 { r - 1 } else { r };
    let re = if r < h - 1 { r + 1 } else { r };
    let cs = if c > 0 { c - 1 } else { c };
    let ce = if c < w - 1 { c + 1 } else { c };
    for j in rs..=re {
      for i in cs..=ce {
        if j == r && i == c { continue; }
        if Self::is_mine(f[j as usize][i as usize]) { n += 1; }
      }
    }
    n
  }

  /// set e
  pub fn set_e(u: &mut u8) -> () { *u |= 0x80; }

  /// is_e
  pub fn is_e(u: u8) -> bool { u & 0x80 != 0 }

  /// set o
  pub fn set_o(u: &mut u8, e: bool) -> () {
    if e && !Self::is_o(*u) { Self::set_e(u); } // force open at ending
    *u |= 0x10;
  }

  /// is_o
  pub fn is_o(u: u8) -> bool { u & 0x10 != 0 }

  /// set m
  pub fn set_m(u: &mut u8) -> () { *u = 0x0f; }

  /// is_mine
  pub fn is_mine(u: u8) -> bool { u == 0x0f }

  /// get v
  pub fn get_v(u: u8) -> u8 { u & 0x0f }
}

/// msg
pub fn msg(x: u16, y: u16, t: time::Instant) -> String {
  format!("({}, {}) {:?}", x, y, t.elapsed())
}

/// show status
pub fn show_status(tm: &mut Termioff, m: &Termine, t: time::Instant) ->
  Result<(), Box<dyn Error>> {
  tm.wr(1, tm.h - 2, 1, Rgb(192, 192, 192), Rgb(8, 8, 8),
    &msg(m.m, m.s & 0x3fff, t))?;
  Ok(())
}

/// main
pub fn main() -> Result<(), Box<dyn Error>> {
  let mut tm = Termioff::new(2)?;
  tm.begin()?;

  let t = time::Instant::now();
  tm.wr(1, tm.h, 3, color::Magenta, Rgb(240, 192, 32), &msg(tm.w, tm.h, t))?;

  // let mut m = Termine::new(1, 1, 0);
  // let mut m = Termine::new(1, 1, 1);
  // let mut m = Termine::new(2, 2, 0);
  // let mut m = Termine::new(2, 2, 2);
  // let mut m = Termine::new(5, 4, 3);
  // let mut m = Termine::new(8, 8, 5);
  let mut m = Termine::new(16, 8, 12);
  // let mut m = Termine::new(80, 50, 12);
  m.reset_t(&mut tm)?;

  let (_tx, rx) = tm.prepare_thread()?;
  loop {
    // thread::sleep(ms);
    match rx.recv_timeout(m.ms) {
    Err(mpsc::RecvTimeoutError::Disconnected) => break, // not be arrived here
    Err(mpsc::RecvTimeoutError::Timeout) => { // idle
      show_status(&mut tm, &m, t)?;
      m.tick(&mut tm)?;
    },
    Ok(ev) => {
      match ev {
      Ok(Event::Key(k)) => {
        match k {
        Key::Ctrl('c') | Key::Char('q') => break,
        Key::Esc | Key::Char('\x1b') => break,
        _ => { if m.update(k) { m.reset_t(&mut tm)?; } }
        }
        if m.is_end() { m.ending(&mut tm)?; break; }
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

  show_status(&mut tm, &m, t)?;
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
