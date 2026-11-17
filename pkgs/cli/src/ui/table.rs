use super::width::string_width;

pub type CellStyle<'a> = Box<dyn Fn(&str, usize, usize) -> String + 'a>;

pub struct TableOptions<'a> {
    pub indent: &'a str,
    pub gap: &'a str,
    pub right: &'a [usize],
    pub head: Option<Vec<String>>,
    pub head_style: Box<dyn Fn(&str) -> String + 'a>,
    pub style: CellStyle<'a>,
}

impl Default for TableOptions<'_> {
    fn default() -> Self {
        TableOptions {
            indent: "  ",
            gap: "  ",
            right: &[],
            head: None,
            head_style: Box::new(|l| l.to_string()),
            style: Box::new(|t, _, _| t.to_string()),
        }
    }
}

pub fn table(rows: &[Vec<String>], opts: TableOptions) -> Vec<String> {
    let mut all: Vec<&Vec<String>> = Vec::new();
    if let Some(h) = &opts.head {
        all.push(h);
    }
    all.extend(rows.iter());
    let columns = all.iter().map(|r| r.len()).max().unwrap_or(0);
    let widths: Vec<usize> = (0..columns)
        .map(|c| all.iter().map(|r| r.get(c).map_or(0, |t| string_width(t))).max().unwrap_or(0))
        .collect();
    let pad = |text: &str, c: usize| " ".repeat(widths[c].saturating_sub(string_width(text)));

    let mut lines = Vec::new();
    if let Some(head) = &opts.head {
        let cells: Vec<String> = head
            .iter()
            .enumerate()
            .map(|(c, h)| {
                if opts.right.contains(&c) {
                    format!("{}{h}", pad(h, c))
                } else {
                    format!("{h}{}", pad(h, c))
                }
            })
            .collect();
        lines.push(format!(
            "{}{}",
            opts.indent,
            (opts.head_style)(cells.join(opts.gap).trim_end())
        ));
    }
    for (r, row) in rows.iter().enumerate() {
        let cells: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(c, text)| {
                let last = c == row.len() - 1;
                if opts.right.contains(&c) {
                    return format!("{}{}", pad(text, c), (opts.style)(text, r, c));
                }
                format!(
                    "{}{}",
                    (opts.style)(text, r, c),
                    if last { String::new() } else { pad(text, c) }
                )
            })
            .collect();
        lines.push(format!("{}{}", opts.indent, cells.join(opts.gap).trim_end()));
    }
    lines
}
