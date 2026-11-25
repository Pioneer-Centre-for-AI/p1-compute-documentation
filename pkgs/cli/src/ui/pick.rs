use std::io::Read;
use std::os::fd::AsFd;

use nix::sys::termios::{SetArg, cfmakeraw, tcgetattr, tcsetattr};

use crate::context::Output;

#[derive(Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Enter,
    Cancel,
    Pick(usize),
    Other,
}

pub fn key(bytes: &[u8], rows: &[String]) -> Key {
    match bytes {
        [b'\r'] | [b'\n'] => Key::Enter,
        [0x1b, b'[', b'A'] | [b'k'] => Key::Up,
        [0x1b, b'[', b'B'] | [b'j'] => Key::Down,
        [0x03] | [0x04] | [0x1b] | [b'q'] => Key::Cancel,
        [digit @ b'1'..=b'9'] if usize::from(digit - b'1') < rows.len() => {
            Key::Pick(usize::from(digit - b'1'))
        }
        [letter] => {
            let starts = |row: &String| row.trim_start().as_bytes().first() == Some(letter);
            let mut with = rows.iter().enumerate().filter(|(_, row)| starts(row));
            match (with.next(), with.next()) {
                (Some((i, _)), None) => Key::Pick(i),
                _ => Key::Other,
            }
        }
        _ => Key::Other,
    }
}

pub fn keys(bytes: &[u8], rows: &[String]) -> Vec<Key> {
    let mut found = Vec::new();
    let mut rest = bytes;
    while !rest.is_empty() {
        let arrow = rest.len() >= 3 && rest[0] == 0x1b && rest[1] == b'[';
        let (one, after) = rest.split_at(if arrow { 3 } else { 1 });
        found.push(key(one, rows));
        rest = after;
    }
    found
}

fn draw(out: &Output, title: &str, rows: &[String], at: usize, again: bool) {
    let mut text = String::new();
    if again {
        text.push_str(&format!("\x1b[{}A\r\x1b[J", rows.len() + 2));
    }
    text.push_str(&format!("{title}\r\n\r\n"));
    for (i, row) in rows.iter().enumerate() {
        if i == at {
            text.push_str(&format!("\x1b[1;36m› {row}\x1b[0m\r\n"));
        } else {
            text.push_str(&format!("  {row}\r\n"));
        }
    }
    out.write(&text);
}

pub async fn pick(out: Output, title: String, rows: Vec<String>, start: usize) -> Option<usize> {
    tokio::task::spawn_blocking(move || {
        let stdin = std::io::stdin();
        let before = tcgetattr(stdin.as_fd()).ok()?;
        let mut raw = before.clone();
        cfmakeraw(&mut raw);
        tcsetattr(stdin.as_fd(), SetArg::TCSANOW, &raw).ok()?;
        out.write("\x1b[?25l");
        let mut at = start.min(rows.len().saturating_sub(1));
        draw(&out, &title, &rows, at, false);
        let chosen = 'asked: loop {
            let mut bytes = [0u8; 64];
            let read = stdin.lock().read(&mut bytes).unwrap_or(0);
            if read == 0 {
                break None;
            }
            for key in keys(&bytes[..read], &rows) {
                match key {
                    Key::Up => at = at.saturating_sub(1),
                    Key::Down => at = (at + 1).min(rows.len() - 1),
                    Key::Pick(i) => at = i,
                    Key::Cancel => break 'asked None,
                    Key::Enter | Key::Other => {}
                }
                if matches!(key, Key::Enter | Key::Pick(_)) {
                    draw(&out, &title, &rows, at, true);
                    break 'asked Some(at);
                }
            }
            draw(&out, &title, &rows, at, true);
        };
        let _ = tcsetattr(stdin.as_fd(), SetArg::TCSANOW, &before);
        out.write("\x1b[?25h\r\n");
        chosen
    })
    .await
    .ok()
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_reached_by_arrows_numbers_and_first_letters() {
        let rows: Vec<String> = ["small  1 GPU", "medium 1 GPU", "large  2 GPUs"]
            .iter()
            .map(|r| r.to_string())
            .collect();
        assert_eq!(key(b"\r", &rows), Key::Enter);
        assert_eq!(key(b"\x1b[A", &rows), Key::Up);
        assert_eq!(key(b"\x1b[B", &rows), Key::Down);
        assert_eq!(key(b"2", &rows), Key::Pick(1));
        assert_eq!(key(b"4", &rows), Key::Other);
        assert_eq!(key(b"l", &rows), Key::Pick(2));
        assert_eq!(key(b"m", &rows), Key::Pick(1));
        assert_eq!(key(b"x", &rows), Key::Other);
        for leave in [&b"\x03"[..], b"q", b"\x1b"] {
            assert_eq!(key(leave, &rows), Key::Cancel);
        }
        assert_eq!(keys(b"\x1b[B\x1b[Bj\r", &rows), [Key::Down, Key::Down, Key::Down, Key::Enter]);
        assert_eq!(keys(b"\x1b", &rows), [Key::Cancel]);
        let twins = vec!["same a".to_string(), "same b".to_string()];
        assert_eq!(key(b"s", &twins), Key::Other);
    }
}
