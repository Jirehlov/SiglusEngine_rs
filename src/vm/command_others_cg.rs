use super::*;

impl Vm {
    fn cg_arg_is_int(args: &[Prop], index: usize) -> bool {
        matches!(args.get(index).map(|p| &p.value), Some(PropValue::Int(_)))
    }

    fn cg_is_group_code_query_args(args: &[Prop]) -> bool {
        (0..5).all(|idx| Self::cg_arg_is_int(args, idx))
    }

    fn cg_is_group_code_write_args(args: &[Prop]) -> bool {
        Self::cg_is_group_code_query_args(args) && Self::cg_arg_is_int(args, 5)
    }

    fn cg_resolve_subcommand_compat(sub: i32, args: &[Prop]) -> i32 {
        use crate::elm::cgtable::*;
        if !Self::cg_is_group_code_query_args(args) {
            return sub;
        }
        // Toolchain cross-check:
        // siglus_scene_script_utility/const.py currently exports cgtable opcodes only up to 10.
        // Some script packs still encode group-code calls via by-name slots with integer payloads.
        // Keep a compatibility remap to preserve runtime behavior until all packs are normalized.
        match sub {
            ELM_CGTABLE_GET_FLAG_NO_BY_NAME => ELM_CGTABLE_GET_FLAG_NO_BY_GROUP_CODE,
            ELM_CGTABLE_GET_NAME_BY_FLAG_NO => ELM_CGTABLE_GET_NAME_BY_GROUP_CODE,
            ELM_CGTABLE_GET_LOOK_BY_NAME => ELM_CGTABLE_GET_LOOK_BY_GROUP_CODE,
            ELM_CGTABLE_SET_LOOK_BY_NAME if Self::cg_is_group_code_write_args(args) => {
                ELM_CGTABLE_SET_LOOK_BY_GROUP_CODE
            }
            // Additional compatibility:
            // legacy script packs may route group-code extended APIs through old scalar slots.
            ELM_CGTABLE_GET_CG_CNT if Self::cg_arg_is_int(args, 5) => {
                ELM_CGTABLE_GET_ONE_CODE_BY_GROUP_CODE
            }
            ELM_CGTABLE_GET_LOOK_CNT => ELM_CGTABLE_GET_GROUP_MEMBER_CNT,
            ELM_CGTABLE_GET_LOOK_PERCENT => ELM_CGTABLE_GET_GROUP_MEMBER_LOOK_CNT,
            ELM_CGTABLE_SET_ALL_FLAG if Self::cg_is_group_code_write_args(args) => {
                ELM_CGTABLE_SET_GROUP_MEMBER_FLAG
            }
            _ => sub,
        }
    }

    fn cg_get_flag_no_by_name(&mut self, name: &str) -> i32 {
        if let Some(no) = self.cg_name_to_flag.get(name) {
            *no
        } else {
            -1
        }
    }

    fn cg_set_look_by_name(&mut self, name: &str, val: i32) {
        let no = if let Some(existing) = self.cg_name_to_flag.get(name).copied() {
            existing
        } else {
            let next = self.cg_flags.len() as i32;
            self.cg_name_to_flag.insert(name.to_string(), next);
            self.cg_flags.push(0);
            next
        };
        let idx = no as usize;
        if self.cg_flags.len() <= idx {
            self.cg_flags.resize(idx + 1, 0);
        }
        self.cg_flags[idx] = if val != 0 { 1 } else { 0 };
    }

    fn cg_group_match_prefix(code: &[i32; 5], query: [i32; 5], depth: usize) -> bool {
        (0..depth).all(|idx| query[idx] == code[idx])
    }

    fn cg_find_flag_for_list_no(&self, list_no: usize) -> Option<i32> {
        let mut direct = None;
        for flag in self.cg_name_to_flag.values() {
            let idx = (*flag).max(0) as usize;
            if idx == list_no {
                direct = Some(*flag);
                break;
            }
        }
        direct.or_else(|| {
            if list_no < self.cg_flags.len() {
                Some(list_no as i32)
            } else {
                None
            }
        })
    }

    pub(super) fn cg_get_flag_list_from_group_code(&self, query: [i32; 5]) -> Vec<i32> {
        // C++ parity target: tnm_cg_table_data.cpp::get_flag_list_func/get_groupe_tree_pointer_funcfunc
        // - first -1 means "all groups"
        // - first non-negative value and first subsequent -1 means "subtree under that prefix"
        // - no -1 means exact code leaf
        let mut out = Vec::new();
        let depth = query.iter().position(|v| *v < 0).unwrap_or(5);
        if depth == 0 {
            for list_no in 0..self.cg_group_codes.len() {
                if self.cg_code_exist_cnt.get(list_no).copied().unwrap_or(0) <= 0 {
                    continue;
                }
                if let Some(flag) = self.cg_find_flag_for_list_no(list_no) {
                    out.push(flag);
                }
            }
            return out;
        }

        for (list_no, code) in self.cg_group_codes.iter().enumerate() {
            if self.cg_code_exist_cnt.get(list_no).copied().unwrap_or(0) < depth as i32 {
                continue;
            }
            let matched = if depth >= 5 {
                code == &query
            } else {
                Self::cg_group_match_prefix(code, query, depth)
            };
            if !matched {
                continue;
            }
            if let Some(flag) = self.cg_find_flag_for_list_no(list_no) {
                out.push(flag);
            }
        }
        out
    }

    fn cg_find_first_flag_from_group_code(&self, query: [i32; 5]) -> Option<i32> {
        self.cg_get_flag_list_from_group_code(query)
            .into_iter()
            .find(|flag_no| *flag_no >= 0)
    }

    pub(super) fn cg_get_flag_no_from_group_code(&self, query: [i32; 5]) -> i32 {
        self.cg_find_first_flag_from_group_code(query).unwrap_or(-1)
    }

    pub(super) fn cg_get_name_from_group_code(&self, query: [i32; 5]) -> String {
        let flag_no = self.cg_get_flag_no_from_group_code(query);
        self.cg_name_to_flag
            .iter()
            .find(|(_, no)| **no == flag_no)
            .map(|(name, _)| name.clone())
            .unwrap_or_default()
    }

    pub(super) fn cg_get_flag_value_from_group_code(&self, query: [i32; 5]) -> Option<i32> {
        let flag_no = self.cg_get_flag_no_from_group_code(query);
        if flag_no < 0 {
            None
        } else {
            self.cg_flags.get(flag_no as usize).copied()
        }
    }

    pub(super) fn cg_set_flag_value_from_group_code(&mut self, query: [i32; 5], value: i32) -> i32 {
        let flag_no = self.cg_get_flag_no_from_group_code(query);
        if flag_no < 0 {
            return -1;
        }
        let idx = flag_no as usize;
        if self.cg_flags.len() <= idx {
            self.cg_flags.resize(idx + 1, 0);
        }
        self.cg_flags[idx] = if value != 0 { 1 } else { 0 };
        flag_no
    }

    pub(super) fn cg_get_one_code_value_from_group_code(
        &self,
        query: [i32; 5],
        code_no: i32,
    ) -> i32 {
        let flag_no = self.cg_get_flag_no_from_group_code(query);
        if flag_no < 0 || !(0..=4).contains(&code_no) {
            return -1;
        }
        self.cg_group_codes
            .get(flag_no as usize)
            .map(|codes| codes[code_no as usize])
            .unwrap_or(-1)
    }

    #[allow(dead_code)]
    pub(super) fn cg_get_all_code_value_from_group_code(&self, query: [i32; 5]) -> Vec<i32> {
        let flag_no = self.cg_get_flag_no_from_group_code(query);
        if flag_no < 0 {
            return Vec::new();
        }
        self.cg_group_codes
            .get(flag_no as usize)
            .map(|codes| codes.to_vec())
            .unwrap_or_default()
    }

    pub(super) fn cg_get_group_member_cnt(&self, query: [i32; 5]) -> i32 {
        let depth = query.iter().position(|v| *v < 0).unwrap_or(5);
        if depth >= 5 {
            return 0;
        }
        let next_depth = depth;
        let mut uniq = std::collections::BTreeSet::new();
        for (list_no, code) in self.cg_group_codes.iter().enumerate() {
            if self.cg_code_exist_cnt.get(list_no).copied().unwrap_or(0) <= next_depth as i32 {
                continue;
            }
            if depth > 0 && !Self::cg_group_match_prefix(code, query, depth) {
                continue;
            }
            uniq.insert(code[next_depth]);
        }
        uniq.len() as i32
    }

    fn cg_group_code_from_args(args: &[Prop]) -> [i32; 5] {
        [
            Self::arg_int(args, 0),
            Self::arg_int(args, 1),
            Self::arg_int(args, 2),
            Self::arg_int(args, 3),
            Self::arg_int(args, 4),
        ]
    }

    fn cg_bit_get(&self, bit: i32, index: i32) -> i32 {
        let idx = index.max(0) as usize;
        let word = self.cg_flags.get(idx / 32).copied().unwrap_or(0);
        let shift = ((idx as i32) % (32 / bit.max(1))).max(0) as usize;
        let mask = (1i32 << bit) - 1;
        (word >> (shift * bit as usize)) & mask
    }

    fn cg_bit_set(&mut self, bit: i32, index: i32, value: i32) {
        let idx = index.max(0) as usize;
        let word_idx = idx / 32;
        if self.cg_flags.len() <= word_idx {
            self.cg_flags.resize(word_idx + 1, 0);
        }
        let shift = ((idx as i32) % (32 / bit.max(1))).max(0) as usize;
        let shift_bits = shift * bit as usize;
        let mask = ((1i32 << bit) - 1) << shift_bits;
        let v = (value & ((1i32 << bit) - 1)) << shift_bits;
        self.cg_flags[word_idx] = (self.cg_flags[word_idx] & !mask) | v;
    }

    fn try_cg_flag_int_list(
        &mut self,
        element: &[i32],
        args: &[Prop],
        bit: i32,
        host: &mut dyn Host,
    ) {
        if element.is_empty() {
            return;
        }
        let sub = Self::cg_resolve_subcommand_compat(element[0], args);
        match sub {
            x if x == crate::elm::ELM_ARRAY => {
                let idx = element.get(1).copied().unwrap_or(0);
                let max_index = if bit == 32 {
                    self.cg_flags.len()
                } else {
                    self.cg_flags.len() * ((32 / bit.max(1)) as usize)
                };
                let out = idx < 0 || (idx as usize) >= max_index;
                if out {
                    if self.options.disp_out_of_range_error {
                        host.on_error_fatal("cgtable.flag index out of range");
                    }
                    if args.is_empty() {
                        self.stack.push_int(0);
                    }
                    return;
                }
                if args.is_empty() {
                    if bit == 32 {
                        self.stack
                            .push_int(self.cg_flags.get(idx.max(0) as usize).copied().unwrap_or(0));
                    } else {
                        self.stack.push_int(self.cg_bit_get(bit, idx));
                    }
                } else if bit == 32 {
                    let u = idx.max(0) as usize;
                    if self.cg_flags.len() <= u {
                        self.cg_flags.resize(u + 1, 0);
                    }
                    self.cg_flags[u] = Self::arg_int(args, 0);
                } else {
                    self.cg_bit_set(bit, idx, Self::arg_int(args, 0));
                }
            }
            x if x == crate::elm::list::ELM_INTLIST_BIT => {
                self.try_cg_flag_int_list(&element[1..], args, 1, host);
            }
            x if x == crate::elm::list::ELM_INTLIST_BIT2 => {
                self.try_cg_flag_int_list(&element[1..], args, 2, host);
            }
            x if x == crate::elm::list::ELM_INTLIST_BIT4 => {
                self.try_cg_flag_int_list(&element[1..], args, 4, host);
            }
            x if x == crate::elm::list::ELM_INTLIST_BIT8 => {
                self.try_cg_flag_int_list(&element[1..], args, 8, host);
            }
            x if x == crate::elm::list::ELM_INTLIST_BIT16 => {
                self.try_cg_flag_int_list(&element[1..], args, 16, host);
            }
            x if x == crate::elm::list::ELM_INTLIST_GET_SIZE => {
                let scale = (32 / bit.max(1)).max(1);
                self.stack.push_int((self.cg_flags.len() as i32) * scale);
            }
            x if x == crate::elm::list::ELM_INTLIST_RESIZE => {
                let size = Self::arg_int(args, 0).max(0);
                let words = if bit == 32 {
                    size
                } else {
                    (size + (32 / bit) - 1) / (32 / bit)
                };
                self.cg_flags.resize(words.max(0) as usize, 0);
            }
            x if x == crate::elm::list::ELM_INTLIST_INIT => {
                self.cg_flags.fill(0);
            }
            x if x == crate::elm::list::ELM_INTLIST_CLEAR => {
                let b = Self::arg_int(args, 0);
                let e = Self::arg_int(args, 1);
                let fill = if args.is_empty() {
                    0
                } else {
                    Self::arg_int(args, 2)
                };
                let scale = (32 / bit.max(1)).max(1) as usize;
                let max_index = if bit == 32 {
                    self.cg_flags.len()
                } else {
                    self.cg_flags.len() * scale
                };
                for idx in b..=e {
                    let out = idx < 0 || (idx as usize) >= max_index;
                    if out {
                        if self.options.disp_out_of_range_error {
                            host.on_error_fatal("cgtable.flag clear index out of range");
                        }
                        continue;
                    }
                    if bit == 32 {
                        self.cg_flags[idx as usize] = fill;
                    } else {
                        self.cg_bit_set(bit, idx, fill);
                    }
                }
            }
            x if x == crate::elm::list::ELM_INTLIST_SETS => {
                let mut idx = Self::arg_int(args, 0);
                let scale = (32 / bit.max(1)).max(1) as usize;
                let max_index = if bit == 32 {
                    self.cg_flags.len()
                } else {
                    self.cg_flags.len() * scale
                };
                for arg in args.iter().skip(1) {
                    let v = match arg.value {
                        PropValue::Int(v) => v,
                        _ => 0,
                    };
                    let out = idx < 0 || (idx as usize) >= max_index;
                    if out {
                        if self.options.disp_out_of_range_error {
                            host.on_error_fatal("cgtable.flag sets index out of range");
                        }
                        idx += 1;
                        continue;
                    }
                    if bit == 32 {
                        self.cg_flags[idx as usize] = v;
                    } else {
                        self.cg_bit_set(bit, idx, v);
                    }
                    idx += 1;
                }
            }
            _ => host.on_error_fatal("無効なコマンドが指定されました。(cgtable.flag)"),
        }
    }

    /// Route `global.cgtable.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_cg_table(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::cgtable::*;
        if element.is_empty() {
            return true;
        }
        match element[0] {
            ELM_CGTABLE_FLAG => {
                if element.len() <= 1 {
                    return true;
                }
                self.try_cg_flag_int_list(&element[1..], args, 32, host);
                true
            }
            ELM_CGTABLE_SET_DISABLE => {
                self.cg_table_off_flag = true;
                true
            }
            ELM_CGTABLE_SET_ENABLE => {
                self.cg_table_off_flag = false;
                true
            }
            ELM_CGTABLE_SET_ALL_FLAG => {
                let v = if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                self.cg_flags.fill(v);
                true
            }
            ELM_CGTABLE_GET_CG_CNT => {
                let cnt = self
                    .cg_get_flag_list_from_group_code([-1, -1, -1, -1, -1])
                    .len() as i32;
                self.stack.push_int(if cnt > 0 {
                    cnt
                } else {
                    self.cg_flags.len() as i32
                });
                true
            }
            ELM_CGTABLE_GET_LOOK_CNT => {
                let candidates = self.cg_get_flag_list_from_group_code([-1, -1, -1, -1, -1]);
                let looked = if candidates.is_empty() {
                    self.cg_flags.iter().filter(|v| **v != 0).count() as i32
                } else {
                    candidates
                        .into_iter()
                        .filter(|flag_no| {
                            self.cg_flags
                                .get((*flag_no).max(0) as usize)
                                .copied()
                                .unwrap_or(0)
                                != 0
                        })
                        .count() as i32
                };
                self.stack.push_int(looked);
                true
            }
            ELM_CGTABLE_GET_LOOK_PERCENT => {
                let candidates = self.cg_get_flag_list_from_group_code([-1, -1, -1, -1, -1]);
                let (all, look) = if candidates.is_empty() {
                    (
                        self.cg_flags.len() as i32,
                        self.cg_flags.iter().filter(|v| **v != 0).count() as i32,
                    )
                } else {
                    let all = candidates.len() as i32;
                    let look = candidates
                        .into_iter()
                        .filter(|flag_no| {
                            self.cg_get_flag_value_from_group_code([-1, -1, -1, -1, -1])
                                .unwrap_or_else(|| {
                                    self.cg_flags
                                        .get((*flag_no).max(0) as usize)
                                        .copied()
                                        .unwrap_or(0)
                                })
                                != 0
                        })
                        .count() as i32;
                    (all, look)
                };
                let percent = if all <= 0 { 0 } else { (look * 100) / all };
                self.stack.push_int(percent);
                true
            }
            ELM_CGTABLE_GET_FLAG_NO_BY_NAME => {
                let name = Self::arg_str_others(args, 0);
                let no = self.cg_get_flag_no_by_name(&name);
                self.stack.push_int(no);
                true
            }
            ELM_CGTABLE_GET_LOOK_BY_NAME => {
                let no = self.cg_get_flag_no_by_name(&Self::arg_str_others(args, 0));
                if no < 0 {
                    self.stack.push_int(-1);
                } else {
                    self.stack
                        .push_int(self.cg_flags.get(no as usize).copied().unwrap_or(0));
                }
                true
            }
            ELM_CGTABLE_SET_LOOK_BY_NAME => {
                self.cg_set_look_by_name(&Self::arg_str_others(args, 0), Self::arg_int(args, 1));
                true
            }
            ELM_CGTABLE_GET_NAME_BY_FLAG_NO => {
                let no = Self::arg_int(args, 0);
                let found = self
                    .cg_name_to_flag
                    .iter()
                    .find(|(_, v)| **v == no)
                    .map(|(k, _)| k.clone())
                    .unwrap_or_default();
                self.stack.push_str(found);
                true
            }
            ELM_CGTABLE_GET_FLAG_NO_BY_GROUP_CODE => {
                self.stack.push_int(
                    self.cg_get_flag_no_from_group_code(Self::cg_group_code_from_args(args)),
                );
                true
            }
            ELM_CGTABLE_GET_NAME_BY_GROUP_CODE => {
                self.stack.push_str(
                    self.cg_get_name_from_group_code(Self::cg_group_code_from_args(args)),
                );
                true
            }
            ELM_CGTABLE_GET_LOOK_BY_GROUP_CODE => {
                self.stack.push_int(
                    self.cg_get_flag_value_from_group_code(Self::cg_group_code_from_args(args))
                        .unwrap_or(-1),
                );
                true
            }
            ELM_CGTABLE_SET_LOOK_BY_GROUP_CODE => {
                self.cg_set_flag_value_from_group_code(
                    Self::cg_group_code_from_args(args),
                    Self::arg_int(args, 5),
                );
                true
            }
            ELM_CGTABLE_GET_ONE_CODE_BY_GROUP_CODE => {
                self.stack
                    .push_int(self.cg_get_one_code_value_from_group_code(
                        Self::cg_group_code_from_args(args),
                        Self::arg_int(args, 5),
                    ));
                true
            }
            ELM_CGTABLE_GET_GROUP_MEMBER_CNT => {
                self.stack
                    .push_int(self.cg_get_group_member_cnt(Self::cg_group_code_from_args(args)));
                true
            }
            ELM_CGTABLE_GET_GROUP_MEMBER_LOOK_CNT => {
                let flags =
                    self.cg_get_flag_list_from_group_code(Self::cg_group_code_from_args(args));
                let looked = flags
                    .into_iter()
                    .filter(|flag_no| {
                        self.cg_flags
                            .get((*flag_no).max(0) as usize)
                            .copied()
                            .unwrap_or(0)
                            != 0
                    })
                    .count() as i32;
                self.stack.push_int(looked);
                true
            }
            ELM_CGTABLE_SET_GROUP_MEMBER_FLAG => {
                let flags =
                    self.cg_get_flag_list_from_group_code(Self::cg_group_code_from_args(args));
                let v = if Self::arg_int(args, 5) != 0 { 1 } else { 0 };
                for flag_no in flags {
                    let idx = flag_no.max(0) as usize;
                    if self.cg_flags.len() <= idx {
                        self.cg_flags.resize(idx + 1, 0);
                    }
                    self.cg_flags[idx] = v;
                }
                true
            }
            _ => {
                host.on_error_fatal("無効なコマンドが指定されました。(cgtable)");
                true
            }
        }
    }
}
