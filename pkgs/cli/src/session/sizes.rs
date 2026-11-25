use serde_json::json;

use crate::cli::args::{SIZE_CPUS, SIZES};
use crate::context::Ctx;
use crate::session::lsf::{QUEUE, gigabytes, parse_minutes};
use crate::ui::style::make_style;
use crate::ui::table::{TableOptions, table};

fn sizes() -> Vec<(&'static str, u32, f64, u32)> {
    SIZES
        .iter()
        .map(|s| {
            let memory = gigabytes(s.mem_per_cpu).unwrap_or(0.0) * f64::from(SIZE_CPUS);
            (s.name, s.gpus, memory, parse_minutes(s.time).unwrap_or(0))
        })
        .collect()
}

fn cells(name: &str, gpus: u32, memory: f64, minutes: u32) -> Vec<String> {
    vec![
        name.to_string(),
        if gpus == 1 { "1 GPU".into() } else { format!("{gpus} GPUs") },
        format!("{SIZE_CPUS} cores"),
        format!("{memory:.0} GB"),
        format!("{} h", minutes / 60),
    ]
}

pub fn choices(with_login: bool) -> Vec<String> {
    let login = ["login", "no GPU", "shared", "light work", "12 h"].map(String::from).to_vec();
    let rows: Vec<Vec<String>> = with_login
        .then_some(login)
        .into_iter()
        .chain(sizes().iter().map(|(n, g, m, t)| cells(n, *g, *m, *t)))
        .collect();
    table(&rows, TableOptions { indent: "", gap: "   ", ..Default::default() })
}

pub fn list(ctx: &Ctx, as_json: bool) -> i32 {
    let rows = sizes();
    if as_json {
        let sizes: Vec<_> = rows
            .iter()
            .enumerate()
            .map(|(i, (name, gpus, memory, minutes))| {
                json!({
                    "name": name,
                    "gpus": gpus,
                    "cpus": SIZE_CPUS,
                    "memoryGb": memory,
                    "minutes": minutes,
                    "default": i == 0,
                })
            })
            .collect();
        ctx.out.write(&format!("{:#}\n", json!({ "queue": QUEUE, "sizes": sizes })));
        return 0;
    }
    let s = make_style(&ctx.env, &ctx.out);
    let cells: Vec<Vec<String>> = rows
        .iter()
        .enumerate()
        .map(|(i, (name, gpus, memory, minutes))| {
            let mut row = cells(name, *gpus, *memory, *minutes);
            row.push(if i == 0 { s.dim("default") } else { String::new() });
            row
        })
        .collect();
    let lines = table(
        &cells,
        TableOptions {
            gap: "   ",
            style: Box::new(|t, _, c| if c == 0 { s.cyan(t) } else { t.to_string() }),
            ..Default::default()
        },
    );
    ctx.out.write(&format!(
        "{}  {}\n\n{}\n\n{}\n",
        s.bold("Sizes"),
        s.dim(&format!("of a --gpu session on {QUEUE}")),
        lines.join("\n"),
        s.dim("aicentre marimo --gpu --size medium asks for one. --time, --cpus and --mem\nchange one part of it. --login runs on the login node, with no GPU.")
    ));
    0
}
