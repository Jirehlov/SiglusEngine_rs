use super::*;
include!("host_stage_impl.rs");
pub(super) fn parse_stage_object_command(element: &[i32]) -> Option<(StagePlane, i32, i32)> {
    use siglus::elm::ELM_ARRAY;

    fn global_to_plane(v: i32) -> Option<StagePlane> {
        if v == siglus::elm::global::ELM_GLOBAL_BACK {
            Some(StagePlane::Back)
        } else if v == siglus::elm::global::ELM_GLOBAL_FRONT {
            Some(StagePlane::Front)
        } else if v == siglus::elm::global::ELM_GLOBAL_NEXT {
            Some(StagePlane::Next)
        } else {
            None
        }
    }

    let (plane, tail) = if let Some(plane) = element.first().and_then(|v| global_to_plane(*v)) {
        (plane, &element[1..])
    } else if element.len() >= 3 && element[0] == siglus::elm::global::ELM_GLOBAL_STAGE && element[1] == ELM_ARRAY {
        let plane = match element[2] {
            0 => StagePlane::Back,
            1 => StagePlane::Front,
            2 => StagePlane::Next,
            _ => return None,
        };
        (plane, &element[3..])
    } else {
        return None;
    };

    if tail.len() >= 4 && tail[0] == siglus::elm::objectlist::ELM_STAGE_OBJECT && tail[1] == ELM_ARRAY {
        return Some((plane, tail[2], tail[3]));
    }

    if tail.len() >= 4 && tail[0] == ELM_ARRAY && tail[2] == siglus::elm::objectlist::ELM_STAGE_OBJECT {
        return Some((plane, tail[1], tail[3]));
    }

    if tail.len() >= 5
        && tail[0] == ELM_ARRAY
        && tail[2] == siglus::elm::objectlist::ELM_STAGE_OBJECT
        && tail[3] == ELM_ARRAY
    {
        return Some((plane, tail[1], tail[4]));
    }

    // Some scripts encode stage/object chains with slightly different ARRAY placement.
    // Be permissive and decode the first recognizable object index + command pair.
    for (i, token) in tail.iter().enumerate() {
        if *token != siglus::elm::objectlist::ELM_STAGE_OBJECT {
            continue;
        }
        if i + 3 < tail.len() && tail[i + 1] == ELM_ARRAY {
            return Some((plane, tail[i + 2], tail[i + 3]));
        }
        if i + 2 < tail.len() {
            return Some((plane, tail[i + 1], tail[i + 2]));
        }
        if i >= 2 && i + 1 < tail.len() && tail[i - 2] == ELM_ARRAY {
            return Some((plane, tail[i - 1], tail[i + 1]));
        }
    }

    None
}

fn is_object_file_create_command(cmd: i32) -> bool {
    // IDs from upstream Siglus element table (vendor/const.py, cmd_object.cpp).
    // These commands all accept file_name as first argument and create/change drawable assets.
    matches!(cmd, 38 | 43 | 45 | 53 | 129 | 132)
}

pub(super) fn parse_stage_plane_command(element: &[i32]) -> Option<(StagePlane, i32)> {
    use siglus::elm::ELM_ARRAY;

    if element.len() >= 2 {
        let plane = if element[0] == siglus::elm::global::ELM_GLOBAL_BACK {
            Some(StagePlane::Back)
        } else if element[0] == siglus::elm::global::ELM_GLOBAL_FRONT {
            Some(StagePlane::Front)
        } else if element[0] == siglus::elm::global::ELM_GLOBAL_NEXT {
            Some(StagePlane::Next)
        } else {
            None
        };
        if let Some(p) = plane {
            return Some((p, element[1]));
        }
    }

    if element.len() >= 4
        && element[0] == siglus::elm::global::ELM_GLOBAL_STAGE
        && element[1] == ELM_ARRAY
        && element[3] != siglus::elm::objectlist::ELM_STAGE_OBJECT
    {
        let plane = match element[2] {
            0 => StagePlane::Back,
            1 => StagePlane::Front,
            2 => StagePlane::Next,
            _ => return None,
        };
        return Some((plane, element[3]));
    }

    None
}

fn default_host_object_state() -> HostObjectState {
    HostObjectState {
        file_name: String::new(),
        pat_no: 0,
        x: 0.0,
        y: 0.0,
        center_x: 0.0,
        center_y: 0.0,
        visible: true,
        order: 0,
        layer: 0,
        scale_x: 1.0,
        scale_y: 1.0,
        rotate_z_deg: 0.0,
        alpha: 1.0,
        alpha_blend: true,
        dst_clip_use: false,
        dst_clip_left: 0.0,
        dst_clip_top: 0.0,
        dst_clip_right: 0.0,
        dst_clip_bottom: 0.0,
        src_clip_use: false,
        src_clip_left: 0.0,
        src_clip_top: 0.0,
        src_clip_right: 0.0,
        src_clip_bottom: 0.0,
        color_rate: 1.0,
        color_r: 1.0,
        color_g: 1.0,
        color_b: 1.0,
        color_add_r: 0.0,
        color_add_g: 0.0,
        color_add_b: 0.0,
        bright: 0.0,
        dark: 0.0,
        mono: 0.0,
        reverse: false,
        seq: 0,
    }
}

pub(super) fn parse_stage_object_prop(element: &[i32]) -> Option<(StagePlane, i32, i32)> {
    parse_stage_object_command(element)
}

pub(super) fn summarize_props(args: &[siglus::vm::Prop]) -> String {
    let mut out = Vec::with_capacity(args.len());
    for p in args.iter().take(8) {
        let s = match &p.value {
            siglus::vm::PropValue::Int(v) => format!("int:{}", v),
            siglus::vm::PropValue::Str(v) => {
                let mut t = v.clone();
                if t.len() > 48 {
                    t.truncate(48);
                    t.push('…');
                }
                format!("str:\"{}\"", t)
            }
            siglus::vm::PropValue::Element(v) => format!("elm:{:?}", v),
            siglus::vm::PropValue::List(v) => format!("list:{}", v.len()),
            siglus::vm::PropValue::IntList(v) => format!("intlist:{}", v.len()),
            siglus::vm::PropValue::StrList(v) => format!("strlist:{}", v.len()),
        };
        out.push(s);
    }
    if args.len() > 8 {
        out.push(format!("...+{}", args.len() - 8));
    }
    format!("[{}]", out.join(", "))
}

pub(super) fn is_visual_or_flow_command(element: &[i32]) -> bool {
        element.iter().any(|v| {
        matches!(
            *v,
            siglus::elm::global::ELM_GLOBAL_STAGE
                | siglus::elm::global::ELM_GLOBAL_BACK
                | siglus::elm::global::ELM_GLOBAL_FRONT
                | siglus::elm::global::ELM_GLOBAL_NEXT
                | siglus::elm::global::ELM_GLOBAL_JUMP
                | siglus::elm::global::ELM_GLOBAL_TIMEWAIT
                | siglus::elm::global::ELM_GLOBAL_TIMEWAIT_KEY
                | siglus::elm::objectlist::ELM_STAGE_OBJECT
        )
    })
}

pub(super) fn looks_like_stage_object_path(element: &[i32]) -> bool {
    use siglus::elm::ELM_ARRAY;
    if element.len() >= 3
        && matches!(element[0], x if x == siglus::elm::global::ELM_GLOBAL_BACK || x == siglus::elm::global::ELM_GLOBAL_FRONT || x == siglus::elm::global::ELM_GLOBAL_NEXT)
        && (element[1] == siglus::elm::objectlist::ELM_STAGE_OBJECT || element[1] == ELM_ARRAY)
    {
        return true;
    }

    element.len() >= 4
        && element[0] == siglus::elm::global::ELM_GLOBAL_STAGE
        && element[1] == ELM_ARRAY
        && (element[3] == siglus::elm::objectlist::ELM_STAGE_OBJECT
            || (element.len() >= 5 && element[4] == siglus::elm::objectlist::ELM_STAGE_OBJECT))
}

fn apply_color_semantics(image: &image::DynamicImage, st: &HostObjectState) -> image::DynamicImage {
    let mut rgba = image.to_rgba8();
    for p in rgba.pixels_mut() {
        let mut r = p[0] as f32;
        let mut g = p[1] as f32;
        let mut b = p[2] as f32;

        if st.mono > 0.0 {
            let y = 0.299 * r + 0.587 * g + 0.114 * b;
            let t = st.mono;
            r = r + (y - r) * t;
            g = g + (y - g) * t;
            b = b + (y - b) * t;
        }

        if st.reverse {
            r = 255.0 - r;
            g = 255.0 - g;
            b = 255.0 - b;
        }

        r = (r + st.bright - st.dark).clamp(0.0, 255.0);
        g = (g + st.bright - st.dark).clamp(0.0, 255.0);
        b = (b + st.bright - st.dark).clamp(0.0, 255.0);

        r = (r * st.color_rate * st.color_r + st.color_add_r).clamp(0.0, 255.0);
        g = (g * st.color_rate * st.color_g + st.color_add_g).clamp(0.0, 255.0);
        b = (b * st.color_rate * st.color_b + st.color_add_b).clamp(0.0, 255.0);

        p[0] = r as u8;
        p[1] = g as u8;
        p[2] = b as u8;
    }
    image::DynamicImage::ImageRgba8(rgba)
}

fn copy_object_state_preserve_seq(dst: &mut HostObjectState, src: &HostObjectState) {
    let seq = dst.seq;
    *dst = src.clone();
    dst.seq = seq;
}

fn reset_object_state_preserve_seq(st: &mut HostObjectState) {
    let seq = st.seq;
    *st = default_host_object_state();
    st.seq = seq;
}
