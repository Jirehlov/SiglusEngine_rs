impl GuiApp {
    fn run_quake_reference_validation(&mut self) {
        let Some(path) = self.quake_ref_csv.clone() else {
            return;
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            log::warn!("quake reference csv not readable: {}", path.display());
            return;
        };

        const AVG_ERR_THRESHOLD: f32 = 12.0;
        const MAX_ERR_THRESHOLD: f32 = 48.0;

        let mut rows = 0usize;
        let mut sum_err = 0.0f32;
        let mut max_err = 0.0f32;
        let mut kind_rows = [0usize; 3];
        let mut kind_sum_err = [0.0f32; 3];
        let mut kind_max_err = [0.0f32; 3];
        let mut kind_hit_rows = [0usize; 3];
        let mut order_clip_hits = 0usize;
        let mut order_clip_misses = 0usize;
        let mut stage_ratio_groups = std::collections::BTreeMap::<String, (usize, f32, f32)>::new();
        let mut bucket_counts = [0usize; 6];
        let mut top_outliers: Vec<(f32, usize, i32, f32, i32, i32)> = Vec::new();
        let mut kind_outliers: [Vec<(f32, usize, i32, f32, i32, i32)>; 3] =
            [Vec::new(), Vec::new(), Vec::new()];
        let mut report_lines = vec![format!("quake reference source={}", path.display())];

        for (line_no, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let cols: Vec<&str> = line.split(',').map(|v| v.trim()).collect();
            if cols.len() < 11 {
                report_lines.push(format!("skip line {}: invalid columns", line_no + 1));
                continue;
            }
            let parse_i32 = |idx: usize| cols.get(idx).and_then(|v| v.parse::<i32>().ok());
            let parse_f32 = |idx: usize| cols.get(idx).and_then(|v| v.parse::<f32>().ok());
            let Some(kind_i) = parse_i32(0) else {
                continue;
            };
            let Some(elapsed_ms) = parse_f32(1) else {
                continue;
            };
            let Some(order) = parse_i32(2) else {
                continue;
            };
            let Some(exp_x) = parse_f32(3) else {
                continue;
            };
            let Some(exp_y) = parse_f32(4) else {
                continue;
            };
            let Some(exp_scale) = parse_f32(5) else {
                continue;
            };
            let Some(exp_cx) = parse_f32(6) else {
                continue;
            };
            let Some(exp_cy) = parse_f32(7) else {
                continue;
            };
            let Some(power) = parse_i32(8) else {
                continue;
            };
            let Some(vec) = parse_i32(9) else {
                continue;
            };

            let begin_order = parse_i32(11).unwrap_or(i32::MIN);
            let end_order = parse_i32(12).unwrap_or(i32::MAX);
            let stage_ratio_x = parse_f32(13).unwrap_or(1.0);
            let stage_ratio_y = parse_f32(14).unwrap_or(1.0);
            let center_x = parse_i32(15).unwrap_or(vec);
            let order_hit = order >= begin_order && order <= end_order;
            if order_hit {
                order_clip_hits += 1;
            } else {
                order_clip_misses += 1;
            }

            let req = siglus::vm::VmQuakeRequest {
                sub: 0,
                kind: match kind_i {
                    1 => siglus::vm::VmQuakeKind::Dir,
                    2 => siglus::vm::VmQuakeKind::Zoom,
                    _ => siglus::vm::VmQuakeKind::Vec,
                },
                time_ms: 1000,
                cnt: 1,
                end_cnt: 1,
                begin_order,
                end_order,
                wait_flag: false,
                key_flag: false,
                power,
                vec,
                center_x,
                center_y: parse_i32(10).unwrap_or(0),
            };
            let (x, y, scale, cx, cy) = Self::quake_transform_for_order_at(&req, elapsed_ms, order);
            let x = x * stage_ratio_x;
            let y = y * stage_ratio_y;
            let err = (x - exp_x).abs()
                + (y - exp_y).abs()
                + (scale - exp_scale).abs() * 100.0
                + (cx - exp_cx).abs()
                + (cy - exp_cy).abs();
            let kind_idx = match kind_i {
                1 => 1usize,
                2 => 2usize,
                _ => 0usize,
            };
            rows += 1;
            sum_err += err;
            max_err = max_err.max(err);
            kind_rows[kind_idx] += 1;
            kind_sum_err[kind_idx] += err;
            kind_max_err[kind_idx] = kind_max_err[kind_idx].max(err);
            if err <= MAX_ERR_THRESHOLD {
                kind_hit_rows[kind_idx] += 1;
            }

            let ratio_key = format!("{:.2}x{:.2}", stage_ratio_x, stage_ratio_y);
            let entry = stage_ratio_groups.entry(ratio_key).or_insert((0, 0.0, 0.0));
            entry.0 += 1;
            entry.1 += err;
            entry.2 = entry.2.max(err);

            let bucket = match err {
                v if v < 2.0 => 0,
                v if v < 5.0 => 1,
                v if v < 10.0 => 2,
                v if v < 20.0 => 3,
                v if v < 40.0 => 4,
                _ => 5,
            };
            bucket_counts[bucket] += 1;
            top_outliers.push((err, line_no + 1, kind_i, elapsed_ms, order, power));
            top_outliers.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            if top_outliers.len() > 8 {
                top_outliers.truncate(8);
            }

            kind_outliers[kind_idx].push((err, line_no + 1, kind_i, elapsed_ms, order, power));
            kind_outliers[kind_idx]
                .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            if kind_outliers[kind_idx].len() > 5 {
                kind_outliers[kind_idx].truncate(5);
            }
        }

        if rows == 0 {
            return;
        }

        let avg = sum_err / rows as f32;
        let pass = avg <= AVG_ERR_THRESHOLD && max_err <= MAX_ERR_THRESHOLD;
        report_lines.push(format!(
            "rows={}, avg_err={}, max_err={}, avg_threshold={}, max_threshold={}, pass={}",
            rows, avg, max_err, AVG_ERR_THRESHOLD, MAX_ERR_THRESHOLD, pass
        ));
        report_lines.push(format!(
            "order_clip_hit_ratio={:.6} (hit={}, miss={})",
            order_clip_hits as f32 / (order_clip_hits + order_clip_misses).max(1) as f32,
            order_clip_hits,
            order_clip_misses
        ));
        report_lines
            .push("error_distribution: [0,2),[2,5),[5,10),[10,20),[20,40),[40,+inf)".to_string());
        for (idx, count) in bucket_counts.into_iter().enumerate() {
            let label = match idx {
                0 => "[0,2)",
                1 => "[2,5)",
                2 => "[5,10)",
                3 => "[10,20)",
                4 => "[20,40)",
                _ => "[40,+inf)",
            };
            let bar_len = (count * 24) / rows.max(1);
            report_lines.push(format!("  {} {:>4} {}", label, count, "#".repeat(bar_len)));
        }
        let kind_name = ["vec", "dir", "zoom"];
        report_lines.push("kind_stats:".to_string());
        for idx in 0..3 {
            let cnt = kind_rows[idx];
            let avg = if cnt == 0 {
                0.0
            } else {
                kind_sum_err[idx] / cnt as f32
            };
            let hit_ratio = kind_hit_rows[idx] as f32 / cnt.max(1) as f32;
            report_lines.push(format!(
                "  kind={} rows={} avg_err={} max_err={} hit_ratio(err<={})={:.6}",
                kind_name[idx], cnt, avg, kind_max_err[idx], MAX_ERR_THRESHOLD, hit_ratio
            ));
        }
        report_lines.push("kind_top_outliers:".to_string());
        for idx in 0..3 {
            report_lines.push(format!("  kind={}", kind_name[idx]));
            for (rank, (err, line_no, _, elapsed_ms, order, power)) in
                kind_outliers[idx].iter().enumerate()
            {
                report_lines.push(format!(
                    "    rank={} err={} line={} elapsed_ms={} order={} power={}",
                    rank + 1,
                    err,
                    line_no,
                    elapsed_ms,
                    order,
                    power
                ));
            }
        }
        report_lines.push("stage_ratio_groups:".to_string());
        for (ratio, (cnt, sum, max_v)) in &stage_ratio_groups {
            let avg = if *cnt == 0 { 0.0 } else { *sum / *cnt as f32 };
            report_lines.push(format!(
                "  ratio={} rows={} avg_err={} max_err={}",
                ratio, cnt, avg, max_v
            ));
        }
        report_lines.push("top_outliers:".to_string());
        for (rank, (err, line_no, kind, elapsed_ms, order, power)) in
            top_outliers.iter().enumerate()
        {
            report_lines.push(format!(
                "  rank={} err={} line={} kind={} elapsed_ms={} order={} power={}",
                rank + 1,
                err,
                line_no,
                kind,
                elapsed_ms,
                order,
                power
            ));
        }

        let csv_path = self.quake_ref_report.with_extension("csv");
        let mut csv = String::from(
            "scope,key,rows,avg_err,max_err,extra
",
        );
        csv.push_str(&format!(
            "summary,all,{},{},{},order_clip_hit_ratio={:.6}
",
            rows,
            avg,
            max_err,
            order_clip_hits as f32 / (order_clip_hits + order_clip_misses).max(1) as f32
        ));
        let kind_name = ["vec", "dir", "zoom"];
        for idx in 0..3 {
            let cnt = kind_rows[idx];
            let avg = if cnt == 0 {
                0.0
            } else {
                kind_sum_err[idx] / cnt as f32
            };
            csv.push_str(&format!(
                "kind,{},{},{},{},hit_ratio(err<={})={:.6}
",
                kind_name[idx],
                cnt,
                avg,
                kind_max_err[idx],
                MAX_ERR_THRESHOLD,
                kind_hit_rows[idx] as f32 / cnt.max(1) as f32
            ));
        }
        for (ratio, (cnt, sum, max_v)) in &stage_ratio_groups {
            let avg = if *cnt == 0 { 0.0 } else { *sum / *cnt as f32 };
            csv.push_str(&format!(
                "stage_ratio,{},{},{},{},
",
                ratio, cnt, avg, max_v
            ));
        }
        for (rank, (err, line_no, kind, elapsed_ms, order, power)) in
            top_outliers.iter().enumerate()
        {
            csv.push_str(&format!(
                "outlier,rank{},{},{},{},line={};kind={};elapsed_ms={};order={};power={}
",
                rank + 1,
                1,
                err,
                err,
                line_no,
                kind,
                elapsed_ms,
                order,
                power
            ));
        }
        for idx in 0..3 {
            for (rank, (err, line_no, kind, elapsed_ms, order, power)) in
                kind_outliers[idx].iter().enumerate()
            {
                csv.push_str(&format!(
                    "kind_outlier,{}-rank{},{},{},{},line={};kind={};elapsed_ms={};order={};power={}
",
                    kind_name[idx],
                    rank + 1,
                    1,
                    err,
                    err,
                    line_no,
                    kind,
                    elapsed_ms,
                    order,
                    power
                ));
            }
        }

        let _ = std::fs::write(
            &self.quake_ref_report,
            report_lines.join(
                "
",
            ),
        );
        let _ = std::fs::write(&csv_path, csv);
        log::info!(
            "quake reference validation done: rows={}, pass={}, report={}, csv={}",
            rows,
            pass,
            self.quake_ref_report.display(),
            csv_path.display()
        );
    }
}
