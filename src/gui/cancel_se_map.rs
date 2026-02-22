fn load_cancel_se_map_from_gameexe(path: &Path) -> BTreeMap<i32, String> {
    #[derive(Clone, Copy)]
    struct PatternRule {
        pat: &'static str,
        score: i32,
    }

    #[derive(Clone, Copy)]
    struct FamilyProfile {
        name: &'static str,
        rules: &'static [PatternRule],
    }

    const PROFILE_DEFAULT_RULES: &[PatternRule] = &[
        PatternRule {
            pat: "SEL.CANCEL.SE",
            score: 100,
        },
        PatternRule {
            pat: "GROUP.CANCEL.SE",
            score: 95,
        },
        PatternRule {
            pat: "SYSCOM.SE",
            score: 90,
        },
        PatternRule {
            pat: "SYSTEM.SE",
            score: 85,
        },
        PatternRule {
            pat: "SYS_SE",
            score: 70,
        },
        PatternRule {
            pat: "SE.",
            score: 60,
        },
        PatternRule {
            pat: "CANCEL_SE",
            score: 50,
        },
    ];

    const PROFILE_KEY_RULES: &[PatternRule] = &[
        PatternRule {
            pat: "SEL.CANCEL.SE",
            score: 120,
        },
        PatternRule {
            pat: "SYSTEM.SE",
            score: 100,
        },
        PatternRule {
            pat: "SYSCOM.SE",
            score: 95,
        },
        PatternRule {
            pat: "GROUP.CANCEL.SE",
            score: 90,
        },
        PatternRule {
            pat: "SE.",
            score: 70,
        },
        PatternRule {
            pat: "SYS_SE",
            score: 60,
        },
        PatternRule {
            pat: "CANCEL_SE",
            score: 55,
        },
    ];

    const PROFILE_LEAF_RULES: &[PatternRule] = &[
        PatternRule {
            pat: "GROUP.CANCEL.SE",
            score: 120,
        },
        PatternRule {
            pat: "SEL.CANCEL.SE",
            score: 110,
        },
        PatternRule {
            pat: "SYSCOM.SE",
            score: 95,
        },
        PatternRule {
            pat: "SYSTEM.SE",
            score: 90,
        },
        PatternRule {
            pat: "SYS_SE",
            score: 75,
        },
        PatternRule {
            pat: "SE.",
            score: 65,
        },
        PatternRule {
            pat: "CANCEL_SE",
            score: 50,
        },
    ];

    const PROFILE_DEFAULT: FamilyProfile = FamilyProfile {
        name: "default",
        rules: PROFILE_DEFAULT_RULES,
    };
    const PROFILE_KEY: FamilyProfile = FamilyProfile {
        name: "key-like",
        rules: PROFILE_KEY_RULES,
    };
    const PROFILE_LEAF: FamilyProfile = FamilyProfile {
        name: "leaf-like",
        rules: PROFILE_LEAF_RULES,
    };

    fn parse_index_from_key(key: &str) -> Option<i32> {
        key.split(['.', '_', '[', ']', '-'])
            .rev()
            .find_map(|seg| seg.parse::<i32>().ok())
    }

    fn pick_profile(cfg: &siglus::gameexe::GameexeConfig) -> FamilyProfile {
        let game_id = cfg.game_id.as_deref().unwrap_or("").to_ascii_uppercase();
        let game_name = cfg.game_name.as_deref().unwrap_or("").to_ascii_uppercase();
        if game_id.contains("KEY") || game_name.contains("KEY") {
            PROFILE_KEY
        } else if game_id.contains("LEAF") || game_name.contains("LEAF") {
            PROFILE_LEAF
        } else {
            PROFILE_DEFAULT
        }
    }

    fn key_priority(key: &str, profile: FamilyProfile) -> (i32, &'static str) {
        let upper = key.to_ascii_uppercase();
        for rule in profile.rules {
            if upper.contains(rule.pat) {
                return (rule.score, rule.pat);
            }
        }
        if upper.contains("SE") && upper.contains("CANCEL") {
            return (40, "SE+CANCEL");
        }
        (0, "NONE")
    }

    fn report_paths(gameexe_path: &Path) -> (PathBuf, PathBuf) {
        let base = gameexe_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let csv_path = std::env::var_os("SIGLUS_CANCEL_SE_REPORT_CSV")
            .map(PathBuf::from)
            .unwrap_or_else(|| base.join("cancel_se_report.csv"));
        let json_path = std::env::var_os("SIGLUS_CANCEL_SE_REPORT_JSON")
            .map(PathBuf::from)
            .unwrap_or_else(|| base.join("cancel_se_report.json"));
        (csv_path, json_path)
    }

    fn json_escape(v: &str) -> String {
        v.replace('\\', "\\\\").replace('"', "\\\"")
    }

    let Ok(cfg) = siglus::gameexe::read_file(path) else {
        return BTreeMap::new();
    };
    let profile = pick_profile(&cfg);

    let mut scored: BTreeMap<i32, (i32, String, String)> = BTreeMap::new();
    let mut conflict_logs = Vec::new();
    let mut rule_hit_count: BTreeMap<&'static str, usize> = BTreeMap::new();

    for ent in &cfg.entries {
        let key = ent.key.as_str();
        let (priority, matched_rule) = key_priority(key, profile);
        if priority <= 0 {
            continue;
        }
        *rule_hit_count.entry(matched_rule).or_default() += 1;
        let Some(se_no) = parse_index_from_key(key) else {
            continue;
        };
        let Some(se_name) = ent.values.first().cloned().filter(|v| !v.trim().is_empty()) else {
            continue;
        };

        let replace = match scored.get(&se_no) {
            Some((old_priority, old_name, old_key)) => {
                let choose_new =
                    priority > *old_priority || (priority == *old_priority && se_name.len() > old_name.len());
                conflict_logs.push(format!(
                    "cancel_se conflict profile={} se_no={} old=({}:{}, p={}) new=({}:{}, p={}) winner={}",
                    profile.name,
                    se_no,
                    old_key,
                    old_name,
                    old_priority,
                    key,
                    se_name,
                    priority,
                    if choose_new { key } else { old_key }
                ));
                choose_new
            }
            None => true,
        };
        if replace {
            scored.insert(se_no, (priority, se_name, key.to_string()));
        }
    }

    for line in &conflict_logs {
        log::info!("{line}");
    }
    log::info!(
        "cancel_se profile={} mapped={}, rule_hits={:?}",
        profile.name,
        scored.len(),
        rule_hit_count
    );

    let (csv_path, json_path) = report_paths(path);
    let mut csv = String::from("kind,profile,se_no,key,name,priority\n");
    for (se_no, (priority, name, key)) in &scored {
        csv.push_str(&format!(
            "mapping,{},{},\"{}\",\"{}\",{}\n",
            profile.name,
            se_no,
            key.replace('"', "\"\""),
            name.replace('"', "\"\""),
            priority
        ));
    }
    for line in &conflict_logs {
        csv.push_str(&format!(
            "conflict,{},{},\"{}\",\"\",0\n",
            profile.name,
            -1,
            line.replace('"', "\"\"")
        ));
    }
    let _ = std::fs::write(&csv_path, csv);

    let mut json = String::new();
    json.push_str("{\n");
    json.push_str(&format!("  \"profile\": \"{}\",\n", profile.name));
    json.push_str("  \"rule_hits\": {\n");
    for (idx, (rule, cnt)) in rule_hit_count.iter().enumerate() {
        let comma = if idx + 1 == rule_hit_count.len() { "" } else { "," };
        json.push_str(&format!("    \"{}\": {}{}\n", json_escape(rule), cnt, comma));
    }
    json.push_str("  },\n");
    json.push_str("  \"mappings\": [\n");
    for (idx, (se_no, (priority, name, key))) in scored.iter().enumerate() {
        let comma = if idx + 1 == scored.len() { "" } else { "," };
        json.push_str(&format!(
            "    {{\"se_no\": {}, \"key\": \"{}\", \"name\": \"{}\", \"priority\": {}}}{}\n",
            se_no,
            json_escape(key),
            json_escape(name),
            priority,
            comma
        ));
    }
    json.push_str("  ],\n");
    json.push_str("  \"conflicts\": [\n");
    for (idx, line) in conflict_logs.iter().enumerate() {
        let comma = if idx + 1 == conflict_logs.len() { "" } else { "," };
        json.push_str(&format!("    \"{}\"{}\n", json_escape(line), comma));
    }
    json.push_str("  ]\n");
    json.push_str("}\n");
    let _ = std::fs::write(&json_path, json);

    scored.into_iter().map(|(k, (_, v, _))| (k, v)).collect()
}
