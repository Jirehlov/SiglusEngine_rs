use super::*;

pub(super) fn collect_append_dirs(base_dir: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![base_dir.to_path_buf()];
    if let Ok(rd) = std::fs::read_dir(base_dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                dirs.push(p);
            }
        }
    }
    if let Some(root) = base_dir.parent() {
        if let Ok(rd) = std::fs::read_dir(root) {
            for ent in rd.flatten() {
                let p = ent.path();
                if p == base_dir || !p.is_dir() {
                    continue;
                }
                if !dirs.iter().any(|d| d == &p) {
                    dirs.push(p);
                }
            }
        }
    }
    dirs
}

fn search_image_in_child_dirs(
    base: &Path,
    stem: &Path,
    ext: &str,
    attempts: &mut Vec<PathBuf>,
) -> Option<PathBuf> {
    let Ok(rd) = std::fs::read_dir(base) else {
        return None;
    };
    for ent in rd.flatten() {
        let dir = ent.path();
        if !dir.is_dir() {
            continue;
        }

        let direct = dir.join(stem).with_extension(ext);
        attempts.push(direct.clone());
        if let Some(found) =
            resolve_existing_relative_case_insensitive(&dir, &stem.with_extension(ext))
        {
            return Some(found);
        }

        let nested = dir.join("g00").join(stem).with_extension(ext);
        attempts.push(nested.clone());
        if let Some(found) = resolve_existing_relative_case_insensitive(
            &dir,
            &Path::new("g00").join(stem).with_extension(ext),
        ) {
            return Some(found);
        }
    }
    None
}

fn normalize_relative_path(path: &Path) -> PathBuf {
    PathBuf::from(path.to_string_lossy().replace('\\', "/"))
}

fn with_numeric_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut out = path.to_path_buf();
    let name = out
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    if !name.is_empty() {
        out.set_file_name(format!("{name}{suffix}"));
    }
    out
}

pub(super) fn has_trailing_ascii_digits(path: &Path) -> bool {
    path.file_name()
        .map(|n| {
            n.to_string_lossy()
                .chars()
                .last()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

fn resolve_existing_relative_case_insensitive(base: &Path, rel: &Path) -> Option<PathBuf> {
    let rel = normalize_relative_path(rel);
    let mut cur = base.to_path_buf();
    for comp in rel.components() {
        let std::path::Component::Normal(name) = comp else {
            continue;
        };
        let next = cur.join(name);
        if next.exists() {
            cur = next;
            continue;
        }

        let name = name.to_string_lossy();
        let mut found = None;
        let rd = std::fs::read_dir(&cur).ok()?;
        for ent in rd.flatten() {
            if ent
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(&name)
            {
                found = Some(ent.path());
                break;
            }
        }
        cur = found?;
    }
    Some(cur)
}

fn format_attempts_for_log(attempts: &[PathBuf]) -> String {
    const MAX_ATTEMPTS: usize = 8;
    if attempts.is_empty() {
        return "(none)".to_string();
    }
    let mut lines: Vec<String> = attempts
        .iter()
        .take(MAX_ATTEMPTS)
        .map(|p| p.display().to_string())
        .collect();
    if attempts.len() > MAX_ATTEMPTS {
        lines.push(format!("... and {} more", attempts.len() - MAX_ATTEMPTS));
    }
    lines.join(" | ")
}

pub(super) fn load_stage_like_cpp(
    base_dir: &Path,
    append_dirs: &[PathBuf],
    file_name: &str,
    pat_no: usize,
) -> Result<image::DynamicImage> {
    let (resolved, attempts) = resolve_image_path_like_cpp(base_dir, append_dirs, file_name);
    if let Some(path) = resolved {
        return load_image_by_path(&path, pat_no).with_context(|| {
            format!(
                "image parse/decode failed: requested={file_name}, resolved={}, pat_no={pat_no}",
                path.display()
            )
        });
    }

    let attempts_desc = format_attempts_for_log(&attempts);
    anyhow::bail!(
        "image not found: requested={file_name}, base_dir={}, attempts={attempts_desc}",
        base_dir.display()
    )
}

fn resolve_image_path_like_cpp(
    base_dir: &Path,
    append_dirs: &[PathBuf],
    file_name: &str,
) -> (Option<PathBuf>, Vec<PathBuf>) {
    let rel_owned = normalize_relative_path(Path::new(file_name));
    let rel = rel_owned.as_path();
    if rel.is_absolute() && rel.exists() {
        return (Some(rel.to_path_buf()), vec![rel.to_path_buf()]);
    }

    let has_ext = rel.extension().and_then(OsStr::to_str).is_some();
    let ext_candidates: Vec<&str> = if has_ext {
        let mut v = Vec::new();
        if let Some(ext) = rel.extension().and_then(OsStr::to_str) {
            v.push(ext);
        }
        for x in IMAGE_EXT_CANDIDATES {
            if !v.iter().any(|e| e.eq_ignore_ascii_case(x)) {
                v.push(x);
            }
        }
        v
    } else {
        IMAGE_EXT_CANDIDATES.to_vec()
    };

    let mut rel_candidates = vec![rel.to_path_buf()];
    if !has_trailing_ascii_digits(rel) {
        rel_candidates.push(with_numeric_suffix(rel, "00"));
    }

    let mut bases: Vec<PathBuf> = append_dirs.to_vec();
    if !bases.iter().any(|p| p == base_dir) {
        bases.insert(0, base_dir.to_path_buf());
    }

    let mut attempts = Vec::new();
    for base in bases {
        for rel_cand in &rel_candidates {
            let exact_as_is = base.join(rel_cand);
            attempts.push(exact_as_is.clone());
            if let Some(found) = resolve_existing_relative_case_insensitive(&base, rel_cand) {
                return (Some(found), attempts);
            }
        }

        for ext in &ext_candidates {
            for rel_cand in &rel_candidates {
                // Exact relative path first.
                let exact = base.join(rel_cand).with_extension(ext);
                attempts.push(exact.clone());
                if let Some(found) =
                    resolve_existing_relative_case_insensitive(&base, &rel_cand.with_extension(ext))
                {
                    return (Some(found), attempts);
                }

                let mut stem_cand = rel_cand.clone();
                stem_cand.set_extension("");
                if stem_cand.as_os_str().is_empty() {
                    stem_cand = rel_cand.clone();
                }
                // C++-like fallback: inside g00/ subdir.
                let sub = base.join("g00").join(&stem_cand).with_extension(ext);
                attempts.push(sub.clone());
                if let Some(found) = resolve_existing_relative_case_insensitive(
                    &base,
                    &Path::new("g00").join(&stem_cand).with_extension(ext),
                ) {
                    return (Some(found), attempts);
                }

                if let Some(found) =
                    search_image_in_child_dirs(&base, &stem_cand, ext, &mut attempts)
                {
                    return (Some(found), attempts);
                }
            }
        }
    }
    (None, attempts)
}

pub(super) fn parse_two_digit_suffix(stem: &str) -> Option<(String, u32)> {
    if stem.len() < 2 {
        return None;
    }
    let (prefix, tail) = stem.split_at(stem.len() - 2);
    if tail.chars().all(|c| c.is_ascii_digit()) {
        return tail.parse::<u32>().ok().map(|n| (prefix.to_string(), n));
    }
    None
}

fn load_g00_image_with_sequence(path: &Path, pat_no: usize) -> Result<image::DynamicImage> {
    let first = siglus::resource::load_g00_images(path)?;
    if first.cuts.is_empty() {
        anyhow::bail!("g00 has no cuts: {}", path.display());
    }
    if pat_no < first.cuts.len() {
        return Ok(first.cuts[pat_no].clone());
    }

    let stem = path.file_stem().and_then(OsStr::to_str).unwrap_or_default();
    let Some((prefix, mut idx)) = parse_two_digit_suffix(stem) else {
        return Ok(first.cuts[0].clone());
    };

    let ext = path.extension().and_then(OsStr::to_str).unwrap_or("g00");
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut offset = first.cuts.len();

    // C++ behavior is hidden inside C_d3d_album::load_g00, but object PATNO is applied against
    // album texture indices. Emulate likely numbered-sequence layout: foo00.g00, foo01.g00, ...
    while idx < 99 {
        idx += 1;
        let cand = dir.join(format!("{prefix}{idx:02}.{ext}"));
        if !cand.exists() {
            break;
        }
        let next = siglus::resource::load_g00_images(&cand)
            .with_context(|| format!("load sequence g00 {}", cand.display()))?;
        if next.cuts.is_empty() {
            continue;
        }
        if pat_no < offset + next.cuts.len() {
            return Ok(next.cuts[pat_no - offset].clone());
        }
        offset += next.cuts.len();
    }

    Ok(first.cuts[0].clone())
}

fn load_image_by_path(path: &Path, pat_no: usize) -> Result<image::DynamicImage> {
    if path
        .extension()
        .and_then(OsStr::to_str)
        .map(|e| e.eq_ignore_ascii_case("g00"))
        .unwrap_or(false)
    {
        return load_g00_image_with_sequence(path, pat_no);
    }

    let img = image::open(path).with_context(|| format!("decode image {}", path.display()))?;
    Ok(img)
}
