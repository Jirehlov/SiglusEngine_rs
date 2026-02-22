use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct OutlierRow {
    scope: String,
    key: String,
    max_err: f32,
    extra: String,
}

fn parse_report_csv(path: &Path) -> Result<Vec<OutlierRow>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let mut rows = Vec::new();
    for (line_no, line) in content.lines().enumerate() {
        if line_no == 0 {
            continue;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.splitn(6, ',').collect();
        if cols.len() < 6 {
            continue;
        }
        if cols[0] != "kind_outlier" {
            continue;
        }
        let Ok(max_err) = cols[4].parse::<f32>() else {
            continue;
        };
        rows.push(OutlierRow {
            scope: cols[0].to_string(),
            key: cols[1].to_string(),
            max_err,
            extra: cols[5].to_string(),
        });
    }
    Ok(rows)
}

fn split_key_rank(key: &str) -> (&str, i32) {
    if let Some((kind, rank)) = key.split_once("-rank") {
        let rank = rank.parse::<i32>().unwrap_or(0);
        (kind, rank)
    } else {
        (key, 0)
    }
}

fn parse_top_n(args: &[String]) -> usize {
    args.windows(2)
        .find_map(|w| {
            if w[0] == "--top" {
                w[1].parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(5)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "usage: quake_kind_outlier_diff <rust_report.csv> <cpp_report.csv> [--top N] [--out FILE]"
        );
        std::process::exit(2);
    }

    let top_n = parse_top_n(&args);
    let out_path = args
        .windows(2)
        .find_map(|w| (w[0] == "--out").then(|| w[1].clone()));

    let rust_rows = match parse_report_csv(Path::new(&args[1])) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let cpp_rows = match parse_report_csv(Path::new(&args[2])) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let mut rust_map = BTreeMap::<(String, i32), OutlierRow>::new();
    let mut cpp_map = BTreeMap::<(String, i32), OutlierRow>::new();

    for row in rust_rows {
        let (kind, rank) = split_key_rank(&row.key);
        if rank >= 1 && rank as usize <= top_n {
            rust_map.insert((kind.to_string(), rank), row);
        }
    }
    for row in cpp_rows {
        let (kind, rank) = split_key_rank(&row.key);
        if rank >= 1 && rank as usize <= top_n {
            cpp_map.insert((kind.to_string(), rank), row);
        }
    }

    let mut out = String::from(
        "kind,rank,rust_scope,cpp_scope,rust_err,cpp_err,delta_err,delta_abs,rust_extra,cpp_extra\n",
    );
    for kind in ["vec", "dir", "zoom"] {
        for rank in 1..=top_n as i32 {
            let rust = rust_map.get(&(kind.to_string(), rank));
            let cpp = cpp_map.get(&(kind.to_string(), rank));
            let rust_err = rust.map(|r| r.max_err).unwrap_or(0.0);
            let cpp_err = cpp.map(|r| r.max_err).unwrap_or(0.0);
            let delta = rust_err - cpp_err;
            let delta_abs = delta.abs();
            let rust_scope = rust.map(|r| r.scope.as_str()).unwrap_or("missing");
            let cpp_scope = cpp.map(|r| r.scope.as_str()).unwrap_or("missing");
            let rust_extra = rust.map(|r| r.extra.as_str()).unwrap_or("missing");
            let cpp_extra = cpp.map(|r| r.extra.as_str()).unwrap_or("missing");
            out.push_str(&format!(
                "{kind},{rank},{rust_scope},{cpp_scope},{rust_err:.6},{cpp_err:.6},{delta:.6},{delta_abs:.6},{rust_extra},{cpp_extra}\n"
            ));
        }
    }

    match out_path {
        Some(path) => {
            if let Err(e) = fs::write(&path, out) {
                eprintln!("failed to write {path}: {e}");
                std::process::exit(1);
            }
            println!("wrote quake kind outlier diff: {path}");
        }
        None => {
            print!("{out}");
        }
    }
}
