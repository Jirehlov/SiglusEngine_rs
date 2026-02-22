use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct Stat {
    count: usize,
    argc_counts: BTreeMap<usize, usize>,
    min_args: Option<usize>,
    max_args: usize,
    examples: Vec<String>,
    source_hits: BTreeMap<String, usize>,
    selector_hist: BTreeMap<i32, usize>,
}

#[derive(Clone, Default)]
struct MatrixRow {
    total_calls: usize,
    selector_hist: String,
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) {
    if root.is_file() {
        out.push(root.to_path_buf());
        return;
    }
    let Ok(rd) = fs::read_dir(root) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            collect_files(&p, out);
        } else if matches!(
            p.extension().and_then(|v| v.to_str()),
            Some("txt") | Some("log")
        ) {
            out.push(p);
        }
    }
}

fn parse_argc(line: &str) -> usize {
    line.split("argc=")
        .nth(1)
        .and_then(|r| r.split_whitespace().next())
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
}

fn parse_args_preview(line: &str) -> String {
    if let Some(s) = line.split("args=").nth(1) {
        if let Some(args) = s.split(" named=").next() {
            return args.trim().to_string();
        }
    }
    "[]".to_string()
}

fn parse_selector_from_preview(preview: &str) -> Option<i32> {
    let start = preview.find("int(")? + 4;
    let end = preview[start..].find(')')? + start;
    preview[start..end].trim().parse::<i32>().ok()
}

fn key_from_line(line: &str) -> Option<&'static str> {
    if line.contains("object.__iapp_dummy") {
        Some("object.__iapp_dummy")
    } else if line.contains("global.__iapp_dummy2_str") {
        Some("global.__iapp_dummy2_str")
    } else if line.contains("global.__iapp_dummy2") {
        Some("global.__iapp_dummy2")
    } else if line.contains("global.__iapp_dummy_str") {
        Some("global.__iapp_dummy_str")
    } else if line.contains("global.__iapp_dummy") {
        Some("global.__iapp_dummy")
    } else {
        None
    }
}

fn parse_csv_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' {
            if in_quotes && i + 1 < chars.len() && chars[i + 1] == '"' {
                cur.push('"');
                i += 2;
                continue;
            }
            in_quotes = !in_quotes;
        } else if ch == ',' && !in_quotes {
            fields.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(ch);
        }
        i += 1;
    }
    fields.push(cur.trim().to_string());
    fields
}

fn parse_baseline(path: &Path) -> BTreeMap<String, MatrixRow> {
    let mut out = BTreeMap::new();
    let Ok(text) = fs::read_to_string(path) else {
        return out;
    };
    let mut selector_col = None;
    for (idx, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cols = parse_csv_fields(line);
        if idx == 0 {
            selector_col = cols.iter().position(|c| c == "selector_hist");
            continue;
        }
        if cols.len() < 2 {
            continue;
        }
        let selector_hist = selector_col
            .and_then(|col| cols.get(col).cloned())
            .unwrap_or_default();
        out.insert(
            cols[0].to_string(),
            MatrixRow {
                total_calls: cols[1].parse().unwrap_or(0),
                selector_hist,
            },
        );
    }
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .unwrap_or_else(|| "/tmp/scene_extract".to_string());
    let output = args
        .next()
        .unwrap_or_else(|| "reference/iapp_dummy_call_matrix.csv".to_string());
    let baseline = args.next();

    let mut files = Vec::new();
    collect_files(Path::new(&input), &mut files);

    let mut stats = BTreeMap::<String, Stat>::new();
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let source = path
            .strip_prefix(&input)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        for line in text.lines() {
            let Some(key) = key_from_line(line) else {
                continue;
            };
            let argc = parse_argc(line);
            let preview = parse_args_preview(line);
            let st = stats.entry(key.to_string()).or_default();
            st.count += 1;
            *st.argc_counts.entry(argc).or_default() += 1;
            st.min_args = Some(st.min_args.map(|v| v.min(argc)).unwrap_or(argc));
            st.max_args = st.max_args.max(argc);
            *st.source_hits.entry(source.clone()).or_default() += 1;
            if key == "object.__iapp_dummy" {
                if let Some(selector) = parse_selector_from_preview(&preview) {
                    *st.selector_hist.entry(selector).or_default() += 1;
                }
            }
            if st.examples.len() < 8 && !st.examples.iter().any(|v| v == &preview) {
                st.examples.push(preview);
            }
        }
    }

    let mut csv = String::from(
        "name,total_calls,min_args,max_args,argc_hist,selector_hist,source_hits,example_args\n",
    );
    let keys = [
        "object.__iapp_dummy",
        "global.__iapp_dummy",
        "global.__iapp_dummy_str",
        "global.__iapp_dummy2",
        "global.__iapp_dummy2_str",
    ];
    let mut current_rows = BTreeMap::<String, MatrixRow>::new();
    for name in keys {
        let st = stats.get(name);
        let total = st.map(|v| v.count).unwrap_or(0);
        let min_args = st.and_then(|v| v.min_args).unwrap_or(0);
        let max_args = st.map(|v| v.max_args).unwrap_or(0);
        let hist = st
            .map(|v| {
                v.argc_counts
                    .iter()
                    .map(|(k, c)| format!("{}:{}", k, c))
                    .collect::<Vec<_>>()
                    .join("|")
            })
            .unwrap_or_default();
        let selector_hist = st
            .map(|v| {
                v.selector_hist
                    .iter()
                    .map(|(k, c)| format!("{}:{}", k, c))
                    .collect::<Vec<_>>()
                    .join("|")
            })
            .unwrap_or_default();
        let source_hits = st
            .map(|v| {
                v.source_hits
                    .iter()
                    .map(|(k, c)| format!("{}:{}", k, c))
                    .collect::<Vec<_>>()
                    .join("|")
            })
            .unwrap_or_else(|| "N/A".to_string());
        let ex = st
            .map(|v| v.examples.join(" || "))
            .unwrap_or_else(|| "N/A".to_string())
            .replace('"', "\"\"");

        current_rows.insert(
            name.to_string(),
            MatrixRow {
                total_calls: total,
                selector_hist: selector_hist.clone(),
            },
        );

        csv.push_str(&format!(
            "{},{} ,{} ,{},\"{}\",\"{}\",\"{}\",\"{}\"\n",
            name, total, min_args, max_args, hist, selector_hist, source_hits, ex
        ));
    }

    if let Some(parent) = Path::new(&output).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&output, csv);
    println!("written {}", output);

    if let Some(base_path) = baseline {
        let base_path_ref = Path::new(&base_path);
        let previous_rows = parse_baseline(base_path_ref);
        let diff_path = Path::new(&output).with_extension("diff.md");
        let mut md = String::new();
        md.push_str(&format!(
            "# iapp_dummy 调用矩阵差异\n\n- baseline: {}\n- current: {}\n\n",
            base_path_ref.display(),
            output
        ));
        md.push_str("| name | Δcalls | baseline_calls | current_calls | baseline_selector_hist | current_selector_hist |\n");
        md.push_str("|---|---:|---:|---:|---|---|\n");

        let mut all_keys = BTreeSet::new();
        all_keys.extend(previous_rows.keys().cloned());
        all_keys.extend(current_rows.keys().cloned());
        for name in all_keys {
            let base = previous_rows.get(&name).cloned().unwrap_or_default();
            let curr = current_rows.get(&name).cloned().unwrap_or_default();
            let delta = curr.total_calls as i64 - base.total_calls as i64;
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                name,
                delta,
                base.total_calls,
                curr.total_calls,
                if base.selector_hist.is_empty() {
                    "-"
                } else {
                    &base.selector_hist
                },
                if curr.selector_hist.is_empty() {
                    "-"
                } else {
                    &curr.selector_hist
                }
            ));
        }

        let _ = fs::write(&diff_path, md);
        println!("written {}", diff_path.display());
    }
}
