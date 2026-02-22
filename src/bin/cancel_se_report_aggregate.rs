use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const PROFILES: [&str; 3] = ["default", "key-like", "leaf-like"];

#[derive(Debug, Deserialize)]
struct CancelSeReport {
    #[serde(default)]
    profile: String,
    #[serde(default)]
    rule_hits: BTreeMap<String, i64>,
    #[serde(default)]
    mappings: Vec<serde_json::Value>,
    #[serde(default)]
    conflicts: Vec<String>,
}

#[derive(Debug)]
struct ReportSummary {
    project: String,
    report_path: String,
    profile: String,
    mapping_cnt: usize,
    conflict_cnt: usize,
    score: f64,
    rule_hits: BTreeMap<String, i64>,
}

fn profile_score(mapping_cnt: usize, conflict_cnt: usize) -> f64 {
    mapping_cnt as f64 - conflict_cnt as f64 * 0.75
}

fn collect_report_files(input: &Path, out: &mut Vec<PathBuf>) {
    if input.is_file() {
        out.push(input.to_path_buf());
        return;
    }
    if !input.is_dir() {
        return;
    }
    let Ok(read_dir) = fs::read_dir(input) else {
        return;
    };
    for ent in read_dir.flatten() {
        let path = ent.path();
        if path.is_dir() {
            collect_report_files(&path, out);
            continue;
        }
        if path
            .file_name()
            .is_some_and(|v| v == "cancel_se_report.json")
        {
            out.push(path);
        }
    }
}

fn read_report(path: &Path) -> Option<ReportSummary> {
    let text = fs::read_to_string(path).ok()?;
    let rep: CancelSeReport = serde_json::from_str(&text).ok()?;
    let profile = if PROFILES.contains(&rep.profile.as_str()) {
        rep.profile
    } else {
        "default".to_string()
    };
    let mapping_cnt = rep.mappings.len();
    let conflict_cnt = rep.conflicts.len();
    Some(ReportSummary {
        project: path
            .parent()
            .and_then(Path::file_name)
            .and_then(|v| v.to_str())
            .unwrap_or("unknown")
            .to_string(),
        report_path: path.display().to_string(),
        profile,
        mapping_cnt,
        conflict_cnt,
        score: profile_score(mapping_cnt, conflict_cnt),
        rule_hits: rep.rule_hits,
    })
}

fn parse_cli() -> (Vec<PathBuf>, PathBuf, PathBuf) {
    let mut inputs = Vec::new();
    let mut csv_out = None;
    let mut json_out = None;
    let mut args = std::env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!(
                    "usage: cancel_se_report_aggregate [inputs ...] [--csv <path>] [--json <path>]\n\
                     default input: $SIGLUS_CANCEL_SE_INPUT_DIR or ./reports/cancel_se\n\
                     default outputs: ./reports/cancel_se_aggregate/cancel_se_profile_aggregate.(csv|json)"
                );
                std::process::exit(0);
            }
            "--csv" => {
                if let Some(v) = args.next() {
                    csv_out = Some(PathBuf::from(v));
                }
            }
            "--json" => {
                if let Some(v) = args.next() {
                    json_out = Some(PathBuf::from(v));
                }
            }
            _ => inputs.push(PathBuf::from(arg)),
        }
    }

    let default_input = std::env::var_os("SIGLUS_CANCEL_SE_INPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("reports/cancel_se"));
    if inputs.is_empty() {
        inputs.push(default_input);
    }

    let default_out_dir = std::env::var_os("SIGLUS_CANCEL_SE_OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("reports/cancel_se_aggregate"));
    let csv = csv_out.unwrap_or_else(|| default_out_dir.join("cancel_se_profile_aggregate.csv"));
    let json = json_out.unwrap_or_else(|| default_out_dir.join("cancel_se_profile_aggregate.json"));
    (inputs, csv, json)
}

fn csv_escape(v: &str) -> String {
    v.replace('"', "\"\"")
}

fn main() {
    let (inputs, csv_path, json_path) = parse_cli();
    if let Some(parent) = csv_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Some(parent) = json_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut files = Vec::new();
    for input in &inputs {
        collect_report_files(input, &mut files);
    }
    files.sort();
    files.dedup();

    let reports = files
        .iter()
        .filter_map(|path| read_report(path))
        .collect::<Vec<_>>();

    if reports.is_empty() {
        eprintln!("no reports found");
        std::process::exit(2);
    }

    let mut score_sum = BTreeMap::<String, f64>::new();
    let mut score_cnt = BTreeMap::<String, usize>::new();
    let mut profile_mapping_sum = BTreeMap::<String, usize>::new();
    let mut profile_conflict_sum = BTreeMap::<String, usize>::new();
    let mut rule_hit_sum = BTreeMap::<String, i64>::new();

    for rep in &reports {
        *score_sum.entry(rep.profile.clone()).or_default() += rep.score;
        *score_cnt.entry(rep.profile.clone()).or_default() += 1;
        *profile_mapping_sum.entry(rep.profile.clone()).or_default() += rep.mapping_cnt;
        *profile_conflict_sum.entry(rep.profile.clone()).or_default() += rep.conflict_cnt;
        for (rule, cnt) in &rep.rule_hits {
            *rule_hit_sum.entry(rule.clone()).or_default() += *cnt;
        }
    }

    let mut avg_scores = BTreeMap::<String, f64>::new();
    for p in PROFILES {
        let sum = score_sum.get(p).copied().unwrap_or(0.0);
        let cnt = score_cnt.get(p).copied().unwrap_or(0);
        avg_scores.insert(p.to_string(), if cnt == 0 { 0.0 } else { sum / cnt as f64 });
    }
    let pos_total: f64 = avg_scores.values().map(|v| v.max(0.0)).sum();
    let mut weights = BTreeMap::<String, f64>::new();
    for p in PROFILES {
        let w = if pos_total <= f64::EPSILON {
            1.0 / PROFILES.len() as f64
        } else {
            avg_scores.get(p).copied().unwrap_or(0.0).max(0.0) / pos_total
        };
        weights.insert(p.to_string(), (w * 10000.0).round() / 10000.0);
    }

    let mut csv = String::from("project,report_path,profile,mapping_cnt,conflict_cnt,score\n");
    for rep in &reports {
        csv.push_str(&format!(
            "{},\"{}\",{},{},{},{:.3}\n",
            csv_escape(&rep.project),
            csv_escape(&rep.report_path),
            rep.profile,
            rep.mapping_cnt,
            rep.conflict_cnt,
            rep.score
        ));
    }
    csv.push_str("\nprofile,avg_score,mapping_sum,conflict_sum,conflict_rate\n");
    for p in PROFILES {
        let mapping_sum = profile_mapping_sum.get(p).copied().unwrap_or(0);
        let conflict_sum = profile_conflict_sum.get(p).copied().unwrap_or(0);
        let rate = if mapping_sum == 0 {
            0.0
        } else {
            conflict_sum as f64 / mapping_sum as f64
        };
        csv.push_str(&format!(
            "{},{:.3},{},{},{:.6}\n",
            p,
            avg_scores.get(p).copied().unwrap_or(0.0),
            mapping_sum,
            conflict_sum,
            rate
        ));
    }
    let _ = fs::write(&csv_path, csv);

    let profile_compare = PROFILES
        .iter()
        .map(|p| {
            let mapping_sum = profile_mapping_sum.get(*p).copied().unwrap_or(0);
            let conflict_sum = profile_conflict_sum.get(*p).copied().unwrap_or(0);
            let conflict_rate = if mapping_sum == 0 {
                0.0
            } else {
                conflict_sum as f64 / mapping_sum as f64
            };
            serde_json::json!({
                "profile": p,
                "avg_score": avg_scores.get(*p).copied().unwrap_or(0.0),
                "recommended_weight": weights.get(*p).copied().unwrap_or(0.0),
                "mapping_sum": mapping_sum,
                "conflict_sum": conflict_sum,
                "conflict_rate": conflict_rate,
            })
        })
        .collect::<Vec<_>>();

    let output = serde_json::json!({
        "report_count": reports.len(),
        "profile_compare": profile_compare,
        "rule_hit_heat": rule_hit_sum,
        "recommended_profile_weights": weights,
    });
    let _ = fs::write(
        &json_path,
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()),
    );

    println!(
        "aggregated reports={} csv={} json={}",
        reports.len(),
        csv_path.display(),
        json_path.display()
    );
}
