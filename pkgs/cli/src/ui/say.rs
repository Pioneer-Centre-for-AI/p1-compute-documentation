use crate::context::Ctx;

pub fn printable(text: &str) -> String {
    text.chars().map(|c| if c == '\t' { ' ' } else { c }).filter(|c| !c.is_control()).collect()
}

pub fn fail(ctx: &Ctx, message: &str, hints: &[String]) -> i32 {
    let mut text = format!("aicentre: {message}\n");
    for h in hints {
        text.push_str(&format!("  {h}\n"));
    }
    ctx.err.write(&text);
    1
}
