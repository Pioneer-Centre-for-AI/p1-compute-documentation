use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::live::{LiveArea, Ticker};
use super::style::{Style, make_style};
use crate::check::types::Status;
use crate::context::{Env, Output};

struct Inner {
    style: Style,
    out: Output,
    area: Option<LiveArea>,
    current: Option<(String, Instant)>,
}

impl Inner {
    fn redraw(&mut self) {
        let Some(area) = self.area.as_mut() else { return };
        let frame = area.next_frame();
        let lines = match &self.current {
            None => vec![],
            Some((text, since)) => {
                let secs = since.elapsed().as_secs();
                let took = if secs >= 3 {
                    let t =
                        if secs < 120 { format!("{secs} s") } else { format!("{} min", secs / 60) };
                    self.style.dim(&format!("  {t}"))
                } else {
                    String::new()
                };
                vec![format!("{} {text}{took}", self.style.spinner(frame))]
            }
        };
        area.draw(lines);
    }

    fn print(&mut self, lines: Vec<String>) {
        match self.area.as_mut() {
            Some(area) => {
                area.print(&lines);
                self.redraw();
            }
            None => self.out.write(&lines.iter().map(|l| format!("{l}\n")).collect::<String>()),
        }
    }

    fn finish(&mut self) {
        if let Some((text, _)) = self.current.take()
            && self.area.is_some()
        {
            let line = format!("{} {text}", self.style.symbol(Status::Ok));
            self.print(vec![line]);
        }
    }
}

#[derive(Clone)]
pub struct Steps {
    inner: Arc<Mutex<Inner>>,
    ticker: Arc<Mutex<Option<Ticker>>>,
    pub style: Style,
}

impl Steps {
    pub fn new(env: &Env, out: &Output) -> Steps {
        let style = make_style(env, out);
        let tty = out.is_tty && env.get("TERM").map(String::as_str) != Some("dumb");
        let inner = Arc::new(Mutex::new(Inner {
            style: style.clone(),
            out: out.clone(),
            area: tty.then(|| LiveArea::new(out.clone())),
            current: None,
        }));
        let mut ticker = None;
        if tty {
            {
                let mut i = inner.lock().unwrap();
                i.area.as_mut().unwrap().start();
                i.redraw();
            }
            let shared = inner.clone();
            ticker = Some(Ticker::every(Duration::from_millis(80), move || {
                shared.lock().unwrap().redraw();
            }));
        }
        Steps { inner, ticker: Arc::new(Mutex::new(ticker)), style }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn step(&self, text: &str) {
        let mut i = self.lock();
        i.finish();
        i.current = Some((text.to_string(), Instant::now()));
        if i.area.is_none() {
            let line = format!("{} {text}\n", i.style.symbol(Status::Info));
            i.out.write(&line);
        }
    }

    pub fn note(&self, text: &str) {
        let mut i = self.lock();
        let line = format!("  {}", i.style.dim(text));
        i.print(vec![line]);
    }

    pub fn fail(&self, text: &str, hint: &[String]) {
        let mut i = self.lock();
        i.current = None;
        let mut lines = vec![format!("{} {text}", i.style.symbol(Status::Fail))];
        for h in hint {
            lines.push(if h.starts_with("  ") {
                format!("    {}", i.style.cyan(h))
            } else {
                format!("    {}", i.style.dim(h))
            });
        }
        i.print(lines);
    }

    pub fn done(&self) {
        if let Some(mut t) = self.ticker.lock().unwrap().take() {
            t.stop();
        }
        let mut i = self.lock();
        i.finish();
        if let Some(area) = i.area.as_mut() {
            area.stop();
        }
    }
}
