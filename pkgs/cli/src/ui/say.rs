use crate::context::Ctx;

fn reorders(c: char) -> bool {
    matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

pub fn printable(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\t' { ' ' } else { c })
        .filter(|c| !c.is_control() && !reorders(*c))
        .collect()
}

pub fn printable_lines(text: &str) -> String {
    text.split('\n').map(printable).collect::<Vec<_>>().join("\n")
}

pub fn fail(ctx: &Ctx, message: &str, hints: &[String]) -> i32 {
    let mut text = format!("aicentre: {message}\n");
    for h in hints {
        text.push_str(&format!("  {h}\n"));
    }
    ctx.err.write(&text);
    1
}
