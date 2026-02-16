pub const ELM_SYSTEM_CHECK_ACTIVE: i32 = 0;
pub const ELM_SYSTEM_CHECK_DEBUG_FLAG: i32 = 13;
pub const ELM_SYSTEM_CHECK_DUMMY_FILE_ONCE: i32 = 2;
pub const ELM_SYSTEM_CHECK_FILE_EXIST: i32 = 6;
pub const ELM_SYSTEM_CHECK_FILE_EXIST_SAVE_DIR: i32 = 12;
pub const ELM_SYSTEM_CLEAR_DUMMY_FILE: i32 = 21;
pub const ELM_SYSTEM_DEBUG_MESSAGEBOX_OK: i32 = 7;
pub const ELM_SYSTEM_DEBUG_MESSAGEBOX_OKCANCEL: i32 = 8;
pub const ELM_SYSTEM_DEBUG_MESSAGEBOX_YESNO: i32 = 9;
pub const ELM_SYSTEM_DEBUG_MESSAGEBOX_YESNOCANCEL: i32 = 10;
pub const ELM_SYSTEM_DEBUG_WRITE_LOG: i32 = 11;
pub const ELM_SYSTEM_GET_CALENDAR: i32 = 14;
pub const ELM_SYSTEM_GET_LANGUAGE: i32 = 16;
pub const ELM_SYSTEM_GET_SPEC_INFO_FOR_CHIHAYA_BENCH: i32 = 4;
pub const ELM_SYSTEM_GET_UNIX_TIME: i32 = 15;
pub const ELM_SYSTEM_MESSAGEBOX_OK: i32 = 17;
pub const ELM_SYSTEM_MESSAGEBOX_OKCANCEL: i32 = 18;
pub const ELM_SYSTEM_MESSAGEBOX_YESNO: i32 = 19;
pub const ELM_SYSTEM_MESSAGEBOX_YESNOCANCEL: i32 = 20;
pub const ELM_SYSTEM_OPEN_DIALOG_FOR_CHIHAYA_BENCH: i32 = 3;
pub const ELM_SYSTEM_SHELL_OPEN_FILE: i32 = 1;
pub const ELM_SYSTEM_SHELL_OPEN_WEB: i32 = 5;

pub const ALL: &[i32] = &[
    crate::elm::system::ELM_SYSTEM_CHECK_ACTIVE,
    crate::elm::system::ELM_SYSTEM_SHELL_OPEN_FILE,
    crate::elm::system::ELM_SYSTEM_CHECK_DUMMY_FILE_ONCE,
    crate::elm::system::ELM_SYSTEM_OPEN_DIALOG_FOR_CHIHAYA_BENCH,
    crate::elm::system::ELM_SYSTEM_GET_SPEC_INFO_FOR_CHIHAYA_BENCH,
    crate::elm::system::ELM_SYSTEM_SHELL_OPEN_WEB,
    crate::elm::system::ELM_SYSTEM_CHECK_FILE_EXIST,
    crate::elm::system::ELM_SYSTEM_DEBUG_MESSAGEBOX_OK,
    crate::elm::system::ELM_SYSTEM_DEBUG_MESSAGEBOX_OKCANCEL,
    crate::elm::system::ELM_SYSTEM_DEBUG_MESSAGEBOX_YESNO,
    crate::elm::system::ELM_SYSTEM_DEBUG_MESSAGEBOX_YESNOCANCEL,
    crate::elm::system::ELM_SYSTEM_DEBUG_WRITE_LOG,
    crate::elm::system::ELM_SYSTEM_CHECK_FILE_EXIST_SAVE_DIR,
    crate::elm::system::ELM_SYSTEM_CHECK_DEBUG_FLAG,
    crate::elm::system::ELM_SYSTEM_GET_CALENDAR,
    crate::elm::system::ELM_SYSTEM_GET_UNIX_TIME,
    crate::elm::system::ELM_SYSTEM_GET_LANGUAGE,
    crate::elm::system::ELM_SYSTEM_MESSAGEBOX_OK,
    crate::elm::system::ELM_SYSTEM_MESSAGEBOX_OKCANCEL,
    crate::elm::system::ELM_SYSTEM_MESSAGEBOX_YESNO,
    crate::elm::system::ELM_SYSTEM_MESSAGEBOX_YESNOCANCEL,
    crate::elm::system::ELM_SYSTEM_CLEAR_DUMMY_FILE,
];

pub fn is_any_system_element(elm: i32) -> bool {
    ALL.binary_search(&elm).is_ok()
}

pub fn is_check_active(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_CHECK_ACTIVE
}

pub fn is_check_debug_flag(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_CHECK_DEBUG_FLAG
}

pub fn is_shell_open(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_SHELL_OPEN_FILE
        || elm == crate::elm::system::ELM_SYSTEM_SHELL_OPEN_WEB
}

pub fn is_check_file_exist(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_CHECK_FILE_EXIST
        || elm == crate::elm::system::ELM_SYSTEM_CHECK_FILE_EXIST_SAVE_DIR
}

pub fn is_get_calendar(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_GET_CALENDAR
}

pub fn is_get_unix_time(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_GET_UNIX_TIME
}

pub fn is_get_language(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_GET_LANGUAGE
}

pub fn is_messagebox(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_MESSAGEBOX_OK
        || elm == crate::elm::system::ELM_SYSTEM_MESSAGEBOX_OKCANCEL
        || elm == crate::elm::system::ELM_SYSTEM_MESSAGEBOX_YESNO
        || elm == crate::elm::system::ELM_SYSTEM_MESSAGEBOX_YESNOCANCEL
}

pub fn is_debug_messagebox_ok(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_DEBUG_MESSAGEBOX_OK
}

pub fn is_debug_messagebox(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_DEBUG_MESSAGEBOX_OK
        || elm == crate::elm::system::ELM_SYSTEM_DEBUG_MESSAGEBOX_OKCANCEL
        || elm == crate::elm::system::ELM_SYSTEM_DEBUG_MESSAGEBOX_YESNO
        || elm == crate::elm::system::ELM_SYSTEM_DEBUG_MESSAGEBOX_YESNOCANCEL
}

pub fn is_debug_write_log(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_DEBUG_WRITE_LOG
}

pub fn is_dummy_file_command(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_CHECK_DUMMY_FILE_ONCE
        || elm == crate::elm::system::ELM_SYSTEM_CLEAR_DUMMY_FILE
}

pub fn is_chihaya_bench(elm: i32) -> bool {
    elm == crate::elm::system::ELM_SYSTEM_OPEN_DIALOG_FOR_CHIHAYA_BENCH
        || elm == crate::elm::system::ELM_SYSTEM_GET_SPEC_INFO_FOR_CHIHAYA_BENCH
}
