mod common;

use common::Sandbox;

#[test]
fn help_fits_80_columns_without_colour_when_piped() {
    let sb = Sandbox::new();
    let ctx = sb.ctx();
    let style = aicentre::ui::style::make_style(&ctx.env, &ctx.out);
    for topic in [None, Some("check"), Some("list"), Some("marimo"), Some("push"), Some("gallery")]
    {
        let text = aicentre::cli::help::render_help(topic, &style, "0.1.0");
        assert!(!text.contains("\x1b["));
        for line in text.lines() {
            assert!(line.chars().count() <= 80, "{line}");
        }
    }
}
