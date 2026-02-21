// Counter / Database / CgTable / BgmTable / G00Buf / Mask / File routing
// Aligns with C++ cmd_others.cpp
use super::*;

impl Vm {
    // =========================================================================
    // Counter
    // =========================================================================

    /// Route `global.counter.<sub>` (counter_list level). Returns `true` if handled.
    pub(super) fn try_command_counter_list(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        match element[0] {
            x if x == crate::elm::ELM_ARRAY => {
                // counter[idx].<sub>
                if element.len() > 2 {
                    self.try_command_counter(&element[2..], args, ret_form, host);
                }
                true
            }
            x if x == crate::elm::counterlist::ELM_COUNTERLIST_GET_SIZE => {
                // Stub: return 0 counters
                self.stack.push_int(0);
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(counterlist)");
                true
            }
        }
    }

    fn try_command_counter(
        &mut self,
        element: &[i32],
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) {
        use crate::elm::counter::*;
        if element.is_empty() {
            return;
        }
        let _arg_int = |idx: usize| -> i32 {
            match args.get(idx).map(|p| &p.value) {
                Some(PropValue::Int(v)) => *v,
                _ => 0,
            }
        };
        match element[0] {
            ELM_COUNTER_SET | ELM_COUNTER_RESET | ELM_COUNTER_START | ELM_COUNTER_START_REAL
            | ELM_COUNTER_START_FRAME | ELM_COUNTER_START_FRAME_REAL
            | ELM_COUNTER_START_FRAME_LOOP | ELM_COUNTER_START_FRAME_LOOP_REAL
            | ELM_COUNTER_STOP | ELM_COUNTER_RESUME | ELM_COUNTER_WAIT
            | ELM_COUNTER_WAIT_KEY => {
                // Host callback for future implementation
                host.on_counter_action(element[0], args);
            }
            ELM_COUNTER_GET => {
                // Stub: return 0
                self.stack.push_int(0);
            }
            ELM_COUNTER_CHECK_VALUE | ELM_COUNTER_CHECK_ACTIVE => {
                self.stack.push_int(0);
            }
            _ => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
            }
        }
    }

    // =========================================================================
    // Database
    // =========================================================================

    /// Route `global.database.<sub>` (database_list level). Returns `true` if handled.
    pub(super) fn try_command_database_list(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        match element[0] {
            x if x == crate::elm::ELM_ARRAY => {
                // database[idx].<sub>
                let db_idx = element.get(1).copied().unwrap_or(0);
                if element.len() > 2 {
                    self.try_command_database(db_idx, &element[2..], args, ret_form, host);
                }
                true
            }
            x if x == crate::elm::databaselist::ELM_DATABASELIST_GET_SIZE => {
                // Stub: 0 databases
                self.stack.push_int(0);
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(databaselist)");
                true
            }
        }
    }

    fn try_command_database(
        &mut self,
        _db_idx: i32,
        element: &[i32],
        _args: &[Prop],
        ret_form: i32,
        _host: &mut dyn Host,
    ) {
        use crate::elm::databaselist::*;
        if element.is_empty() {
            return;
        }
        match element[0] {
            ELM_DATABASE_GET_NUM | ELM_DATABASE_CHECK_ITEM | ELM_DATABASE_CHECK_COLUMN
            | ELM_DATABASE_FIND_NUM | ELM_DATABASE_FIND_STR | ELM_DATABASE_FIND_STR_REAL => {
                self.stack.push_int(0);
            }
            ELM_DATABASE_GET_STR | ELM_DATABASE_GET_DATA => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                } else {
                    self.stack.push_str(String::new());
                }
            }
            _ => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                } else if ret_form == crate::elm::form::STR {
                    self.stack.push_str(String::new());
                }
            }
        }
    }

    // =========================================================================
    // CgTable
    // =========================================================================

    /// Route `global.cgtable.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_cg_table(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::cgtable::*;
        if element.is_empty() {
            return true;
        }
        match element[0] {
            ELM_CGTABLE_FLAG => {
                // Delegate to int_list operations on the cg flag array
                // Stub: accept + default return
                if element.len() > 1 {
                    // int_list sub-command on cg flag list
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(0);
                    }
                }
                true
            }
            ELM_CGTABLE_SET_DISABLE | ELM_CGTABLE_SET_ENABLE | ELM_CGTABLE_SET_ALL_FLAG
            | ELM_CGTABLE_SET_LOOK_BY_NAME => {
                // Write operations → accept
                true
            }
            ELM_CGTABLE_GET_CG_CNT | ELM_CGTABLE_GET_LOOK_CNT | ELM_CGTABLE_GET_LOOK_PERCENT
            | ELM_CGTABLE_GET_FLAG_NO_BY_NAME | ELM_CGTABLE_GET_LOOK_BY_NAME => {
                self.stack.push_int(0);
                true
            }
            ELM_CGTABLE_GET_NAME_BY_FLAG_NO => {
                self.stack.push_str(String::new());
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(cgtable)");
                true
            }
        }
    }

    // =========================================================================
    // BgmTable
    // =========================================================================

    /// Route `global.bgmtable.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_bgm_table(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::bgmtable::*;
        if element.is_empty() {
            return true;
        }
        match element[0] {
            ELM_BGMTABLE_GET_BGM_CNT | ELM_BGMTABLE_GET_LISTEN_BY_NAME => {
                self.stack.push_int(0);
                true
            }
            ELM_BGMTABLE_SET_LISTEN_BY_NAME | ELM_BGMTABLE_SET_ALL_FLAG => {
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(bgmtable)");
                true
            }
        }
    }

    // =========================================================================
    // G00Buf / Mask / File — accept + default returns
    // =========================================================================

    /// Route `global.g00buf.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_g00buf(
        &mut self,
        _element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        ret_form: i32,
        _host: &mut dyn Host,
    ) -> bool {
        // All g00buf commands are rendering-layer; accept and return defaults.
        if ret_form == crate::elm::form::INT {
            self.stack.push_int(0);
        } else if ret_form == crate::elm::form::STR {
            self.stack.push_str(String::new());
        }
        true
    }

    /// Route `global.mask.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_mask_list(
        &mut self,
        _element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        ret_form: i32,
        _host: &mut dyn Host,
    ) -> bool {
        // All mask commands are rendering-layer; accept and return defaults.
        if ret_form == crate::elm::form::INT {
            self.stack.push_int(0);
        } else if ret_form == crate::elm::form::STR {
            self.stack.push_str(String::new());
        }
        true
    }

    /// Route `global.file.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_file(
        &mut self,
        _element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        ret_form: i32,
        _host: &mut dyn Host,
    ) -> bool {
        // C++ cmd_others.cpp::tnm_command_proc_file — accept + default returns
        if ret_form == crate::elm::form::INT {
            self.stack.push_int(0);
        } else if ret_form == crate::elm::form::STR {
            self.stack.push_str(String::new());
        }
        true
    }
}
