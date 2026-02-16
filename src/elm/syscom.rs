pub const ELM_SYSCOM_CALL_EX: i32 = 236;
pub const ELM_SYSCOM_CHANGE_QUICK_SAVE: i32 = 66;
pub const ELM_SYSCOM_CHANGE_SAVE: i32 = 22;
pub const ELM_SYSCOM_CHECK_AUTO_MODE_ENABLE: i32 = 220;
pub const ELM_SYSCOM_CHECK_INNER_SAVE: i32 = 275;
pub const ELM_SYSCOM_CHECK_JOYPAD_MODE: i32 = 333;
pub const ELM_SYSCOM_CHECK_LOAD_ENABLE: i32 = 262;
pub const ELM_SYSCOM_CHECK_MSG_BACK_ENABLE: i32 = 198;
pub const ELM_SYSCOM_CHECK_READ_SKIP_ENABLE: i32 = 206;
pub const ELM_SYSCOM_CHECK_RETURN_TO_MENU_ENABLE: i32 = 241;
pub const ELM_SYSCOM_CHECK_RETURN_TO_SEL_ENABLE: i32 = 234;
pub const ELM_SYSCOM_CHECK_SAVE_ENABLE: i32 = 255;
pub const ELM_SYSCOM_CLEAR_INNER_SAVE: i32 = 276;
pub const ELM_SYSCOM_COPY_INNER_SAVE: i32 = 274;
pub const ELM_SYSCOM_COPY_QUICK_SAVE: i32 = 128;
pub const ELM_SYSCOM_COPY_SAVE: i32 = 67;
pub const ELM_SYSCOM_DELETE_QUICK_SAVE: i32 = 65;
pub const ELM_SYSCOM_DELETE_SAVE: i32 = 19;
pub const ELM_SYSCOM_GET_AUTO_MODE_ONOFF_FLAG: i32 = 215;
pub const ELM_SYSCOM_GET_CURRENT_SAVE_MESSAGE: i32 = 295;
pub const ELM_SYSCOM_GET_CURRENT_SAVE_SCENE_TITLE: i32 = 294;
pub const ELM_SYSCOM_GET_HIDE_MWND_ENABLE_FLAG: i32 = 224;
pub const ELM_SYSCOM_GET_LOCAL_EXTRA_MODE_EXIST_FLAG: i32 = 63;
pub const ELM_SYSCOM_GET_QUICK_SAVE_CNT: i32 = 168;
pub const ELM_SYSCOM_GET_QUICK_SAVE_DAY: i32 = 173;
pub const ELM_SYSCOM_GET_QUICK_SAVE_EXIST: i32 = 169;
pub const ELM_SYSCOM_GET_QUICK_SAVE_HOUR: i32 = 175;
pub const ELM_SYSCOM_GET_QUICK_SAVE_MESSAGE: i32 = 130;
pub const ELM_SYSCOM_GET_QUICK_SAVE_MILLISECOND: i32 = 178;
pub const ELM_SYSCOM_GET_QUICK_SAVE_MINUTE: i32 = 176;
pub const ELM_SYSCOM_GET_QUICK_SAVE_MONTH: i32 = 172;
pub const ELM_SYSCOM_GET_QUICK_SAVE_NEW_NO: i32 = 170;
pub const ELM_SYSCOM_GET_QUICK_SAVE_SECOND: i32 = 177;
pub const ELM_SYSCOM_GET_QUICK_SAVE_TITLE: i32 = 179;
pub const ELM_SYSCOM_GET_QUICK_SAVE_WEEKDAY: i32 = 174;
pub const ELM_SYSCOM_GET_QUICK_SAVE_YEAR: i32 = 171;
pub const ELM_SYSCOM_GET_SAVE_CNT: i32 = 68;
pub const ELM_SYSCOM_GET_SAVE_DAY: i32 = 72;
pub const ELM_SYSCOM_GET_SAVE_EXIST: i32 = 69;
pub const ELM_SYSCOM_GET_SAVE_HOUR: i32 = 74;
pub const ELM_SYSCOM_GET_SAVE_MESSAGE: i32 = 129;
pub const ELM_SYSCOM_GET_SAVE_MILLISECOND: i32 = 77;
pub const ELM_SYSCOM_GET_SAVE_MINUTE: i32 = 75;
pub const ELM_SYSCOM_GET_SAVE_MONTH: i32 = 71;
pub const ELM_SYSCOM_GET_SAVE_NEW_NO: i32 = 79;
pub const ELM_SYSCOM_GET_SAVE_SECOND: i32 = 76;
pub const ELM_SYSCOM_GET_SAVE_TITLE: i32 = 78;
pub const ELM_SYSCOM_GET_SAVE_WEEKDAY: i32 = 73;
pub const ELM_SYSCOM_GET_SAVE_YEAR: i32 = 70;
pub const ELM_SYSCOM_INNER_LOAD: i32 = 273;
pub const ELM_SYSCOM_INNER_SAVE: i32 = 272;
pub const ELM_SYSCOM_LOAD: i32 = 256;
pub const ELM_SYSCOM_OPEN_MSG_BACK: i32 = 192;
pub const ELM_SYSCOM_OPEN_TWEET_DIALOG: i32 = 327;
pub const ELM_SYSCOM_QUICK_LOAD: i32 = 20;
pub const ELM_SYSCOM_QUICK_SAVE: i32 = 18;
pub const ELM_SYSCOM_SAVE: i32 = 249;
pub const ELM_SYSCOM_SET_HIDE_MWND_ONOFF_FLAG: i32 = 221;
pub const ELM_SYSCOM_SET_SE_VOLUME: i32 = 32;
pub const ELM_SYSCOM_SET_NO_WIPE_ANIME_ONOFF: i32 = 113;
pub const ELM_SYSCOM_SET_NO_WIPE_ANIME_ONOFF_DEFAULT: i32 = 114;
pub const ELM_SYSCOM_GET_NO_WIPE_ANIME_ONOFF: i32 = 115;
pub const ELM_SYSCOM_SET_SKIP_WIPE_ANIME_ONOFF: i32 = 116;
pub const ELM_SYSCOM_SET_SKIP_WIPE_ANIME_ONOFF_DEFAULT: i32 = 117;
pub const ELM_SYSCOM_GET_SKIP_WIPE_ANIME_ONOFF: i32 = 118;

pub fn is_query_int(elm: i32) -> bool {
    matches!(
        elm,
        x if x == crate::elm::syscom::ELM_SYSCOM_GET_HIDE_MWND_ENABLE_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_AUTO_MODE_ONOFF_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_LOCAL_EXTRA_MODE_EXIST_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_CNT
            || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_CNT
            || x == crate::elm::syscom::ELM_SYSCOM_CHECK_JOYPAD_MODE
            || x == crate::elm::syscom::ELM_SYSCOM_CHECK_SAVE_ENABLE
            || x == crate::elm::syscom::ELM_SYSCOM_CHECK_LOAD_ENABLE
            || x == crate::elm::syscom::ELM_SYSCOM_CHECK_AUTO_MODE_ENABLE
            || x == crate::elm::syscom::ELM_SYSCOM_CHECK_READ_SKIP_ENABLE
            || x == crate::elm::syscom::ELM_SYSCOM_CHECK_RETURN_TO_SEL_ENABLE
            || x == crate::elm::syscom::ELM_SYSCOM_CHECK_MSG_BACK_ENABLE
            || x == crate::elm::syscom::ELM_SYSCOM_CHECK_RETURN_TO_MENU_ENABLE
            || x == crate::elm::syscom::ELM_SYSCOM_CHECK_MSG_BACK_OPEN
            || x == crate::elm::syscom::ELM_SYSCOM_GET_READ_SKIP_ONOFF_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_READ_SKIP_ENABLE_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_READ_SKIP_EXIST_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_AUTO_MODE_ENABLE_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_AUTO_MODE_EXIST_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_MSG_BACK_ENABLE_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_MSG_BACK_EXIST_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_RETURN_TO_SEL_ENABLE_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_RETURN_TO_SEL_EXIST_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_RETURN_TO_MENU_ENABLE_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_RETURN_TO_MENU_EXIST_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_ENABLE_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_EXIST_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_LOAD_ENABLE_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_LOAD_EXIST_FLAG
            || x == crate::elm::syscom::ELM_SYSCOM_GET_NO_WIPE_ANIME_ONOFF
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SKIP_WIPE_ANIME_ONOFF
    ) || is_slot_exist_query(elm)
        || is_get_new_no(elm)
        || is_slot_time_query(elm)
}

pub fn is_get_new_no(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_NEW_NO
        || elm == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_NEW_NO
}

pub fn is_delete_save(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_DELETE_SAVE
        || elm == crate::elm::syscom::ELM_SYSCOM_DELETE_QUICK_SAVE
}

pub fn is_slot_count_query(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_CNT
        || elm == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_CNT
}

pub fn is_slot_exist_query(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_EXIST
        || elm == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_EXIST
        || elm == crate::elm::syscom::ELM_SYSCOM_CHECK_INNER_SAVE
}

pub fn is_slot_time_query(elm: i32) -> bool {
    matches!(
        elm,
        x if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_YEAR
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_MONTH
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_DAY
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_WEEKDAY
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_HOUR
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_MINUTE
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_SECOND
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_MILLISECOND
            || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_YEAR
            || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_MONTH
            || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_DAY
            || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_WEEKDAY
            || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_HOUR
            || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_MINUTE
            || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_SECOND
            || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_MILLISECOND
    )
}

pub fn is_slot_text_query(elm: i32) -> bool {
    matches!(
        elm,
        x if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_TITLE
            || x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_MESSAGE
            || x == crate::elm::syscom::ELM_SYSCOM_GET_CURRENT_SAVE_SCENE_TITLE
            || x == crate::elm::syscom::ELM_SYSCOM_GET_CURRENT_SAVE_MESSAGE
            || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_TITLE
            || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_MESSAGE
    )
}

pub fn is_save_or_load(elm: i32) -> bool {
    matches!(
        elm,
        x if x == crate::elm::syscom::ELM_SYSCOM_SAVE
            || x == crate::elm::syscom::ELM_SYSCOM_LOAD
            || x == crate::elm::syscom::ELM_SYSCOM_QUICK_SAVE
            || x == crate::elm::syscom::ELM_SYSCOM_QUICK_LOAD
            || x == crate::elm::syscom::ELM_SYSCOM_INNER_SAVE
            || x == crate::elm::syscom::ELM_SYSCOM_INNER_LOAD
            || x == crate::elm::syscom::ELM_SYSCOM_CLEAR_INNER_SAVE
            || x == crate::elm::syscom::ELM_SYSCOM_COPY_INNER_SAVE
            || x == crate::elm::syscom::ELM_SYSCOM_COPY_SAVE
            || x == crate::elm::syscom::ELM_SYSCOM_COPY_QUICK_SAVE
            || x == crate::elm::syscom::ELM_SYSCOM_CHANGE_SAVE
            || x == crate::elm::syscom::ELM_SYSCOM_CHANGE_QUICK_SAVE
    )
}

pub fn is_open_dialog(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_OPEN_TWEET_DIALOG
}

pub fn is_msg_back_dialog_control(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_OPEN_MSG_BACK
        || elm == crate::elm::syscom::ELM_SYSCOM_CLOSE_MSG_BACK
}

pub fn is_call_ex(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_CALL_EX
}
pub fn is_set_hide_mwnd_flag(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_SET_HIDE_MWND_ONOFF_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_HIDE_MWND_ENABLE_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_HIDE_MWND_EXIST_FLAG
}
pub fn is_set_se_volume(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_SET_SE_VOLUME
}

pub fn is_set_wipe_anime_onoff(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_SET_NO_WIPE_ANIME_ONOFF
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_NO_WIPE_ANIME_ONOFF_DEFAULT
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_SKIP_WIPE_ANIME_ONOFF
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_SKIP_WIPE_ANIME_ONOFF_DEFAULT
}

pub fn is_set_enable_flag(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_SET_READ_SKIP_ENABLE_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_READ_SKIP_ONOFF_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_READ_SKIP_EXIST_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_AUTO_MODE_ENABLE_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_AUTO_MODE_ONOFF_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_AUTO_MODE_EXIST_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_MSG_BACK_ENABLE_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_MSG_BACK_EXIST_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_RETURN_TO_SEL_ENABLE_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_RETURN_TO_SEL_EXIST_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_RETURN_TO_MENU_ENABLE_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_RETURN_TO_MENU_EXIST_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_SAVE_ENABLE_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_SAVE_EXIST_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_LOAD_ENABLE_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_SET_LOAD_EXIST_FLAG
}

pub fn is_hide_mwnd_query(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_GET_HIDE_MWND_ONOFF_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_GET_HIDE_MWND_ENABLE_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_GET_HIDE_MWND_EXIST_FLAG
        || elm == crate::elm::syscom::ELM_SYSCOM_CHECK_HIDE_MWND_ENABLE
}

pub fn is_feature_check_query(elm: i32) -> bool {
    elm == crate::elm::syscom::ELM_SYSCOM_CHECK_READ_SKIP_ENABLE
        || elm == crate::elm::syscom::ELM_SYSCOM_CHECK_AUTO_MODE_ENABLE
        || elm == crate::elm::syscom::ELM_SYSCOM_CHECK_MSG_BACK_ENABLE
        || elm == crate::elm::syscom::ELM_SYSCOM_CHECK_RETURN_TO_SEL_ENABLE
        || elm == crate::elm::syscom::ELM_SYSCOM_CHECK_RETURN_TO_MENU_ENABLE
        || elm == crate::elm::syscom::ELM_SYSCOM_CHECK_SAVE_ENABLE
        || elm == crate::elm::syscom::ELM_SYSCOM_CHECK_LOAD_ENABLE
}

// AUTOGENERATED from siglus_scene_script_utility/const.py SYSTEM_ELEMENT_DEFS
pub const ELM_SYSCOM_CALL_SYSCOM_MENU: i32 = 0; // call_syscom_menu
pub const ELM_SYSCOM_CALL_SAVE_MENU: i32 = 1; // call_save_menu
pub const ELM_SYSCOM_CALL_LOAD_MENU: i32 = 2; // call_load_menu
pub const ELM_SYSCOM_CALL_CONFIG_MENU: i32 = 3; // call_config_menu
pub const ELM_SYSCOM_SET_WINDOW_MODE: i32 = 4; // set_window_mode
pub const ELM_SYSCOM_INIT_SYSCOM_FLAG: i32 = 5; // init_syscom_flag
pub const ELM_SYSCOM_SET_SYSCOM_MENU_ENABLE: i32 = 6; // set_syscom_menu_enable
pub const ELM_SYSCOM_SET_SYSCOM_MENU_DISABLE: i32 = 7; // set_syscom_menu_disable
pub const ELM_SYSCOM_SET_NO_MWND_ANIME_ONOFF: i32 = 8; // set_no_mwnd_anime_onoff
pub const ELM_SYSCOM_GET_WINDOW_MODE: i32 = 9; // get_window_mode
pub const ELM_SYSCOM_GET_SAVELOAD_ALERT_ONOFF: i32 = 10; // get_saveload_alert_onoff
pub const ELM_SYSCOM_SET_MWND_BTN_ENABLE: i32 = 11; // set_mwnd_btn_enable
pub const ELM_SYSCOM_SET_MWND_BTN_DISABLE: i32 = 12; // set_mwnd_btn_disable
pub const ELM_SYSCOM_SET_WINDOW_MODE_SIZE: i32 = 13; // set_window_mode_size
pub const ELM_SYSCOM_SET_GLOBAL_EXTRA_SWITCH_ONOFF: i32 = 14; // set_global_extra_switch_onoff
pub const ELM_SYSCOM_SET_GLOBAL_EXTRA_SWITCH_ONOFF_DEFAULT: i32 = 15; // set_global_extra_switch_onoff_default
pub const ELM_SYSCOM_GET_WINDOW_MODE_SIZE: i32 = 16; // get_window_mode_size
pub const ELM_SYSCOM_GET_GLOBAL_EXTRA_SWITCH_ONOFF: i32 = 17; // get_global_extra_switch_onoff
pub const ELM_SYSCOM_SET_BGM_VOLUME: i32 = 21; // set_bgm_volume
pub const ELM_SYSCOM_SET_LOCAL_EXTRA_MODE_VALUE: i32 = 23; // set_local_extra_mode_value
pub const ELM_SYSCOM_SET_BGM_VOLUME_DEFAULT: i32 = 24; // set_bgm_volume_default
pub const ELM_SYSCOM_GET_BGM_VOLUME: i32 = 25; // get_bgm_volume
pub const ELM_SYSCOM_SET_KOE_VOLUME: i32 = 26; // set_koe_volume
pub const ELM_SYSCOM_SET_KOE_VOLUME_DEFAULT: i32 = 27; // set_koe_volume_default
pub const ELM_SYSCOM_GET_KOE_VOLUME: i32 = 28; // get_koe_volume
pub const ELM_SYSCOM_SET_PCM_VOLUME: i32 = 29; // set_pcm_volume
pub const ELM_SYSCOM_SET_PCM_VOLUME_DEFAULT: i32 = 30; // set_pcm_volume_default
pub const ELM_SYSCOM_GET_PCM_VOLUME: i32 = 31; // get_pcm_volume
pub const ELM_SYSCOM_SET_SE_VOLUME_DEFAULT: i32 = 33; // set_se_volume_default
pub const ELM_SYSCOM_GET_SE_VOLUME: i32 = 34; // get_se_volume
pub const ELM_SYSCOM_SET_BGM_ONOFF: i32 = 35; // set_bgm_onoff
pub const ELM_SYSCOM_SET_KOE_ONOFF: i32 = 36; // set_koe_onoff
pub const ELM_SYSCOM_SET_PCM_ONOFF: i32 = 37; // set_pcm_onoff
pub const ELM_SYSCOM_SET_SE_ONOFF: i32 = 38; // set_se_onoff
pub const ELM_SYSCOM_SET_ALL_VOLUME: i32 = 39; // set_all_volume
pub const ELM_SYSCOM_SET_ALL_VOLUME_DEFAULT: i32 = 40; // set_all_volume_default
pub const ELM_SYSCOM_GET_ALL_VOLUME: i32 = 41; // get_all_volume
pub const ELM_SYSCOM_GET_BGM_ONOFF: i32 = 42; // get_bgm_onoff
pub const ELM_SYSCOM_GET_KOE_ONOFF: i32 = 43; // get_koe_onoff
pub const ELM_SYSCOM_GET_PCM_ONOFF: i32 = 44; // get_pcm_onoff
pub const ELM_SYSCOM_GET_SE_ONOFF: i32 = 45; // get_se_onoff
pub const ELM_SYSCOM_SET_MESSAGE_SPEED: i32 = 46; // set_message_speed
pub const ELM_SYSCOM_SET_MESSAGE_SPEED_DEFAULT: i32 = 47; // set_message_speed_default
pub const ELM_SYSCOM_GET_MESSAGE_SPEED: i32 = 48; // get_message_speed
pub const ELM_SYSCOM_SET_MESSAGE_NOWAIT: i32 = 49; // set_message_nowait
pub const ELM_SYSCOM_GET_MESSAGE_NOWAIT: i32 = 50; // get_message_nowait
pub const ELM_SYSCOM_SET_AUTO_MODE_MOJI_WAIT: i32 = 51; // set_auto_mode_moji_wait
pub const ELM_SYSCOM_SET_AUTO_MODE_MOJI_WAIT_DEFAULT: i32 = 52; // set_auto_mode_moji_wait_default
pub const ELM_SYSCOM_GET_AUTO_MODE_MOJI_WAIT: i32 = 53; // get_auto_mode_moji_wait
pub const ELM_SYSCOM_SET_AUTO_MODE_MIN_WAIT: i32 = 54; // set_auto_mode_min_wait
pub const ELM_SYSCOM_SET_AUTO_MODE_MIN_WAIT_DEFAULT: i32 = 55; // set_auto_mode_min_wait_default
pub const ELM_SYSCOM_GET_AUTO_MODE_MIN_WAIT: i32 = 56; // get_auto_mode_min_wait
pub const ELM_SYSCOM_GET_LOCAL_EXTRA_MODE_VALUE: i32 = 57; // get_local_extra_mode_value
pub const ELM_SYSCOM_SET_LOCAL_EXTRA_MODE_ENABLE_FLAG: i32 = 58; // set_local_extra_mode_enable_flag
pub const ELM_SYSCOM_GET_LOCAL_EXTRA_MODE_ENABLE_FLAG: i32 = 59; // get_local_extra_mode_enable_flag
pub const ELM_SYSCOM_SET_ALL_ONOFF: i32 = 60; // set_all_onoff
pub const ELM_SYSCOM_GET_ALL_ONOFF: i32 = 61; // get_all_onoff
pub const ELM_SYSCOM_SET_LOCAL_EXTRA_MODE_EXIST_FLAG: i32 = 62; // set_local_extra_mode_exist_flag
pub const ELM_SYSCOM_CHECK_LOCAL_EXTRA_MODE_ENABLE: i32 = 64; // check_local_extra_mode_enable
pub const ELM_SYSCOM_SET_SAVELOAD_ALERT_ONOFF: i32 = 80; // set_saveload_alert_onoff
pub const ELM_SYSCOM_GET_NO_MWND_ANIME_ONOFF: i32 = 81; // get_no_mwnd_anime_onoff
pub const ELM_SYSCOM_SET_FILTER_COLOR_R: i32 = 82; // set_filter_color_r
pub const ELM_SYSCOM_SET_FILTER_COLOR_R_DEFAULT: i32 = 83; // set_filter_color_r_default
pub const ELM_SYSCOM_GET_FILTER_COLOR_R: i32 = 84; // get_filter_color_r
pub const ELM_SYSCOM_SET_FILTER_COLOR_G: i32 = 85; // set_filter_color_g
pub const ELM_SYSCOM_SET_FILTER_COLOR_B: i32 = 86; // set_filter_color_b
pub const ELM_SYSCOM_SET_FILTER_COLOR_A: i32 = 87; // set_filter_color_a
pub const ELM_SYSCOM_SET_FILTER_COLOR_G_DEFAULT: i32 = 88; // set_filter_color_g_default
pub const ELM_SYSCOM_SET_FILTER_COLOR_B_DEFAULT: i32 = 89; // set_filter_color_b_default
pub const ELM_SYSCOM_SET_FILTER_COLOR_A_DEFAULT: i32 = 90; // set_filter_color_a_default
pub const ELM_SYSCOM_GET_FILTER_COLOR_G: i32 = 91; // get_filter_color_g
pub const ELM_SYSCOM_GET_FILTER_COLOR_B: i32 = 92; // get_filter_color_b
pub const ELM_SYSCOM_GET_FILTER_COLOR_A: i32 = 93; // get_filter_color_a
pub const ELM_SYSCOM_SET_BGMFADE_VOLUME: i32 = 94; // set_bgmfade_volume
pub const ELM_SYSCOM_SET_BGMFADE_VOLUME_DEFAULT: i32 = 95; // set_bgmfade_volume_default
pub const ELM_SYSCOM_GET_BGMFADE_VOLUME: i32 = 96; // get_bgmfade_volume
pub const ELM_SYSCOM_SET_BGMFADE_ONOFF: i32 = 97; // set_bgmfade_onoff
pub const ELM_SYSCOM_GET_BGMFADE_ONOFF: i32 = 98; // get_bgmfade_onoff
pub const ELM_SYSCOM_SET_WINDOW_MODE_DEFAULT: i32 = 99; // set_window_mode_default
pub const ELM_SYSCOM_SET_WINDOW_MODE_SIZE_DEFAULT: i32 = 100; // set_window_mode_size_default
pub const ELM_SYSCOM_SET_ALL_ONOFF_DEFAULT: i32 = 101; // set_all_onoff_default
pub const ELM_SYSCOM_SET_BGM_ONOFF_DEFAULT: i32 = 102; // set_bgm_onoff_default
pub const ELM_SYSCOM_SET_KOE_ONOFF_DEFAULT: i32 = 103; // set_koe_onoff_default
pub const ELM_SYSCOM_SET_PCM_ONOFF_DEFAULT: i32 = 104; // set_pcm_onoff_default
pub const ELM_SYSCOM_SET_SE_ONOFF_DEFAULT: i32 = 105; // set_se_onoff_default
pub const ELM_SYSCOM_SET_BGMFADE_ONOFF_DEFAULT: i32 = 106; // set_bgmfade_onoff_default
pub const ELM_SYSCOM_SET_MESSAGE_NOWAIT_DEFAULT: i32 = 107; // set_message_nowait_default
pub const ELM_SYSCOM_SET_SAVELOAD_ALERT_ONOFF_DEFAULT: i32 = 108; // set_saveload_alert_onoff_default
pub const ELM_SYSCOM_SET_NO_MWND_ANIME_ONOFF_DEFAULT: i32 = 109; // set_no_mwnd_anime_onoff_default
pub const ELM_SYSCOM_SET_SLEEP_ONOFF: i32 = 110; // set_sleep_onoff
pub const ELM_SYSCOM_SET_SLEEP_ONOFF_DEFAULT: i32 = 111; // set_sleep_onoff_default
pub const ELM_SYSCOM_GET_SLEEP_ONOFF: i32 = 112; // get_sleep_onoff
pub const ELM_SYSCOM_SET_WHEEL_NEXT_MESSAGE_ONOFF: i32 = 119; // set_wheel_next_message_onoff
pub const ELM_SYSCOM_SET_WHEEL_NEXT_MESSAGE_ONOFF_DEFAULT: i32 = 120; // set_wheel_next_message_onoff_default
pub const ELM_SYSCOM_GET_WHEEL_NEXT_MESSAGE_ONOFF: i32 = 121; // get_wheel_next_message_onoff
pub const ELM_SYSCOM_SET_KOE_DONT_STOP_ONOFF: i32 = 122; // set_koe_dont_stop_onoff
pub const ELM_SYSCOM_SET_KOE_DONT_STOP_ONOFF_DEFAULT: i32 = 123; // set_koe_dont_stop_onoff_default
pub const ELM_SYSCOM_GET_KOE_DONT_STOP_ONOFF: i32 = 124; // get_koe_dont_stop_onoff
pub const ELM_SYSCOM_SET_SKIP_UNREAD_MESSAGE_ONOFF: i32 = 125; // set_skip_unread_message_onoff
pub const ELM_SYSCOM_SET_SKIP_UNREAD_MESSAGE_ONOFF_DEFAULT: i32 = 126; // set_skip_unread_message_onoff_default
pub const ELM_SYSCOM_GET_SKIP_UNREAD_MESSAGE_ONOFF: i32 = 127; // get_skip_unread_message_onoff
pub const ELM_SYSCOM_GET_SAVE_COMMENT: i32 = 131; // get_save_comment
pub const ELM_SYSCOM_GET_QUICK_SAVE_COMMENT: i32 = 132; // get_quick_save_comment
pub const ELM_SYSCOM_SET_MWND_BTN_TOUCH_ENABLE: i32 = 133; // set_mwnd_btn_touch_enable
pub const ELM_SYSCOM_SET_MWND_BTN_TOUCH_DISABLE: i32 = 134; // set_mwnd_btn_touch_disable
pub const ELM_SYSCOM_CALL_CONFIG_MESSAGE_SPEED_MENU: i32 = 135; // call_config_message_speed_menu
pub const ELM_SYSCOM_CALL_CONFIG_FILTER_COLOR_MENU: i32 = 136; // call_config_filter_color_menu
pub const ELM_SYSCOM_CALL_CONFIG_BGMFADE_MENU: i32 = 137; // call_config_bgmfade_menu
pub const ELM_SYSCOM_CALL_CONFIG_WINDOW_MODE_MENU: i32 = 138; // call_config_window_mode_menu
pub const ELM_SYSCOM_CALL_CONFIG_VOLUME_MENU: i32 = 139; // call_config_volume_menu
pub const ELM_SYSCOM_CALL_CONFIG_AUTO_MODE_MENU: i32 = 140; // call_config_auto_mode_menu
pub const ELM_SYSCOM_CALL_CONFIG_SYSTEM_MENU: i32 = 141; // call_config_system_menu
pub const ELM_SYSCOM_CALL_CONFIG_FONT_MENU: i32 = 142; // call_config_font_menu
pub const ELM_SYSCOM_SET_CHARAKOE_ONOFF: i32 = 143; // set_charakoe_onoff
pub const ELM_SYSCOM_SET_CHARAKOE_ONOFF_DEFAULT: i32 = 144; // set_charakoe_onoff_default
pub const ELM_SYSCOM_GET_CHARAKOE_ONOFF: i32 = 145; // get_charakoe_onoff
pub const ELM_SYSCOM_CALL_CONFIG_CHARAKOE_MENU: i32 = 146; // call_config_charakoe_menu
pub const ELM_SYSCOM_CALL_CONFIG_KOEMODE_MENU: i32 = 147; // call_config_koemode_menu
pub const ELM_SYSCOM_SET_KOEMODE: i32 = 148; // set_koemode
pub const ELM_SYSCOM_SET_KOEMODE_DEFAULT: i32 = 149; // set_koemode_default
pub const ELM_SYSCOM_GET_KOEMODE: i32 = 150; // get_koemode
pub const ELM_SYSCOM_CALL_CONFIG_JITAN_MENU: i32 = 151; // call_config_jitan_menu
pub const ELM_SYSCOM_SET_JITAN_SPEED: i32 = 152; // set_jitan_speed
pub const ELM_SYSCOM_SET_JITAN_NORMAL_ONOFF: i32 = 153; // set_jitan_normal_onoff
pub const ELM_SYSCOM_SET_JITAN_NORMAL_ONOFF_DEFAULT: i32 = 154; // set_jitan_normal_onoff_default
pub const ELM_SYSCOM_GET_JITAN_NORMAL_ONOFF: i32 = 155; // get_jitan_normal_onoff
pub const ELM_SYSCOM_SET_JITAN_AUTO_MODE_ONOFF: i32 = 156; // set_jitan_auto_mode_onoff
pub const ELM_SYSCOM_SET_JITAN_AUTO_MODE_ONOFF_DEFAULT: i32 = 157; // set_jitan_auto_mode_onoff_default
pub const ELM_SYSCOM_GET_JITAN_AUTO_MODE_ONOFF: i32 = 158; // get_jitan_auto_mode_onoff
pub const ELM_SYSCOM_SET_JITAN_KOE_REPLAY_ONOFF: i32 = 159; // set_jitan_koe_replay_onoff
pub const ELM_SYSCOM_SET_JITAN_KOE_REPLAY_ONOFF_DEFAULT: i32 = 160; // set_jitan_koe_replay_onoff_default
pub const ELM_SYSCOM_GET_JITAN_KOE_REPLAY_ONOFF: i32 = 161; // get_jitan_koe_replay_onoff
pub const ELM_SYSCOM_SET_JITAN_SPEED_DEFAULT: i32 = 162; // set_jitan_speed_default
pub const ELM_SYSCOM_GET_JITAN_SPEED: i32 = 163; // get_jitan_speed
pub const ELM_SYSCOM_SET_GLOBAL_EXTRA_MODE_VALUE: i32 = 164; // set_global_extra_mode_value
pub const ELM_SYSCOM_SET_GLOBAL_EXTRA_MODE_VALUE_DEFAULT: i32 = 165; // set_global_extra_mode_value_default
pub const ELM_SYSCOM_GET_GLOBAL_EXTRA_MODE_VALUE: i32 = 166; // get_global_extra_mode_value
pub const ELM_SYSCOM_CALL_CONFIG_MOVIE_MENU: i32 = 167; // call_config_movie_menu
pub const ELM_SYSCOM_SET_SAVE_COMMENT: i32 = 180; // set_save_comment
pub const ELM_SYSCOM_SET_QUICK_SAVE_COMMENT: i32 = 181; // set_quick_save_comment
pub const ELM_SYSCOM_SET_SAVE_VALUE: i32 = 182; // set_save_value
pub const ELM_SYSCOM_GET_SAVE_VALUE: i32 = 183; // get_save_value
pub const ELM_SYSCOM_GET_QUICK_SAVE_VALUE: i32 = 184; // get_quick_save_value
pub const ELM_SYSCOM_SET_QUICK_SAVE_VALUE: i32 = 185; // set_quick_save_value
pub const ELM_SYSCOM_SET_CHARAKOE_VOLUME: i32 = 186; // set_charakoe_volume
pub const ELM_SYSCOM_SET_CHARAKOE_VOLUME_DEFAULT: i32 = 187; // set_charakoe_volume_default
pub const ELM_SYSCOM_GET_CHARAKOE_VOLUME: i32 = 188; // get_charakoe_volume
pub const ELM_SYSCOM_SET_OBJECT_DISP_ONOFF: i32 = 189; // set_object_disp_onoff
pub const ELM_SYSCOM_SET_OBJECT_DISP_ONOFF_DEFAULT: i32 = 190; // set_object_disp_onoff_default
pub const ELM_SYSCOM_GET_OBJECT_DISP_ONOFF: i32 = 191; // get_object_disp_onoff
pub const ELM_SYSCOM_CLOSE_MSG_BACK: i32 = 193; // close_msg_back
pub const ELM_SYSCOM_SET_MSG_BACK_ENABLE_FLAG: i32 = 194; // set_msg_back_enable_flag
pub const ELM_SYSCOM_GET_MSG_BACK_ENABLE_FLAG: i32 = 195; // get_msg_back_enable_flag
pub const ELM_SYSCOM_SET_MSG_BACK_EXIST_FLAG: i32 = 196; // set_msg_back_exist_flag
pub const ELM_SYSCOM_GET_MSG_BACK_EXIST_FLAG: i32 = 197; // get_msg_back_exist_flag
pub const ELM_SYSCOM_GET_TOTAL_PLAY_TIME: i32 = 199; // get_total_play_time
pub const ELM_SYSCOM_SET_READ_SKIP_ONOFF_FLAG: i32 = 200; // set_read_skip_onoff_flag
pub const ELM_SYSCOM_GET_READ_SKIP_ONOFF_FLAG: i32 = 201; // get_read_skip_onoff_flag
pub const ELM_SYSCOM_SET_READ_SKIP_ENABLE_FLAG: i32 = 202; // set_read_skip_enable_flag
pub const ELM_SYSCOM_GET_READ_SKIP_ENABLE_FLAG: i32 = 203; // get_read_skip_enable_flag
pub const ELM_SYSCOM_SET_READ_SKIP_EXIST_FLAG: i32 = 204; // set_read_skip_exist_flag
pub const ELM_SYSCOM_GET_READ_SKIP_EXIST_FLAG: i32 = 205; // get_read_skip_exist_flag
pub const ELM_SYSCOM_SET_AUTO_SKIP_ONOFF_FLAG: i32 = 207; // set_auto_skip_onoff_flag
pub const ELM_SYSCOM_GET_AUTO_SKIP_ONOFF_FLAG: i32 = 208; // get_auto_skip_onoff_flag
pub const ELM_SYSCOM_SET_AUTO_SKIP_ENABLE_FLAG: i32 = 209; // set_auto_skip_enable_flag
pub const ELM_SYSCOM_GET_AUTO_SKIP_ENABLE_FLAG: i32 = 210; // get_auto_skip_enable_flag
pub const ELM_SYSCOM_SET_AUTO_SKIP_EXIST_FLAG: i32 = 211; // set_auto_skip_exist_flag
pub const ELM_SYSCOM_GET_AUTO_SKIP_EXIST_FLAG: i32 = 212; // get_auto_skip_exist_flag
pub const ELM_SYSCOM_CHECK_AUTO_SKIP_ENABLE: i32 = 213; // check_auto_skip_enable
pub const ELM_SYSCOM_SET_AUTO_MODE_ONOFF_FLAG: i32 = 214; // set_auto_mode_onoff_flag
pub const ELM_SYSCOM_SET_AUTO_MODE_ENABLE_FLAG: i32 = 216; // set_auto_mode_enable_flag
pub const ELM_SYSCOM_GET_AUTO_MODE_ENABLE_FLAG: i32 = 217; // get_auto_mode_enable_flag
pub const ELM_SYSCOM_SET_AUTO_MODE_EXIST_FLAG: i32 = 218; // set_auto_mode_exist_flag
pub const ELM_SYSCOM_GET_AUTO_MODE_EXIST_FLAG: i32 = 219; // get_auto_mode_exist_flag
pub const ELM_SYSCOM_GET_HIDE_MWND_ONOFF_FLAG: i32 = 222; // get_hide_mwnd_onoff_flag
pub const ELM_SYSCOM_SET_HIDE_MWND_ENABLE_FLAG: i32 = 223; // set_hide_mwnd_enable_flag
pub const ELM_SYSCOM_SET_HIDE_MWND_EXIST_FLAG: i32 = 225; // set_hide_mwnd_exist_flag
pub const ELM_SYSCOM_GET_HIDE_MWND_EXIST_FLAG: i32 = 226; // get_hide_mwnd_exist_flag
pub const ELM_SYSCOM_CHECK_HIDE_MWND_ENABLE: i32 = 227; // check_hide_mwnd_enable
pub const ELM_SYSCOM_RETURN_TO_SEL: i32 = 228; // return_to_sel
pub const ELM_SYSCOM_SET_TOTAL_PLAY_TIME: i32 = 229; // set_total_play_time
pub const ELM_SYSCOM_SET_RETURN_TO_SEL_ENABLE_FLAG: i32 = 230; // set_return_to_sel_enable_flag
pub const ELM_SYSCOM_GET_RETURN_TO_SEL_ENABLE_FLAG: i32 = 231; // get_return_to_sel_enable_flag
pub const ELM_SYSCOM_SET_RETURN_TO_SEL_EXIST_FLAG: i32 = 232; // set_return_to_sel_exist_flag
pub const ELM_SYSCOM_GET_RETURN_TO_SEL_EXIST_FLAG: i32 = 233; // get_return_to_sel_exist_flag
pub const ELM_SYSCOM_RETURN_TO_MENU: i32 = 235; // return_to_menu
pub const ELM_SYSCOM_SET_RETURN_TO_MENU_ENABLE_FLAG: i32 = 237; // set_return_to_menu_enable_flag
pub const ELM_SYSCOM_GET_RETURN_TO_MENU_ENABLE_FLAG: i32 = 238; // get_return_to_menu_enable_flag
pub const ELM_SYSCOM_SET_RETURN_TO_MENU_EXIST_FLAG: i32 = 239; // set_return_to_menu_exist_flag
pub const ELM_SYSCOM_GET_RETURN_TO_MENU_EXIST_FLAG: i32 = 240; // get_return_to_menu_exist_flag
pub const ELM_SYSCOM_END_GAME: i32 = 242; // end_game
pub const ELM_SYSCOM_GET_PLAY_SILENT_SOUND_ONOFF: i32 = 243; // get_play_silent_sound_onoff
pub const ELM_SYSCOM_SET_END_GAME_ENABLE_FLAG: i32 = 244; // set_end_game_enable_flag
pub const ELM_SYSCOM_GET_END_GAME_ENABLE_FLAG: i32 = 245; // get_end_game_enable_flag
pub const ELM_SYSCOM_SET_END_GAME_EXIST_FLAG: i32 = 246; // set_end_game_exist_flag
pub const ELM_SYSCOM_GET_END_GAME_EXIST_FLAG: i32 = 247; // get_end_game_exist_flag
pub const ELM_SYSCOM_CHECK_END_GAME_ENABLE: i32 = 248; // check_end_game_enable
pub const ELM_SYSCOM_SET_PLAY_SILENT_SOUND_ONOFF: i32 = 250; // set_play_silent_sound_onoff
pub const ELM_SYSCOM_SET_SAVE_ENABLE_FLAG: i32 = 251; // set_save_enable_flag
pub const ELM_SYSCOM_GET_SAVE_ENABLE_FLAG: i32 = 252; // get_save_enable_flag
pub const ELM_SYSCOM_SET_SAVE_EXIST_FLAG: i32 = 253; // set_save_exist_flag
pub const ELM_SYSCOM_GET_SAVE_EXIST_FLAG: i32 = 254; // get_save_exist_flag
pub const ELM_SYSCOM_SET_PLAY_SILENT_SOUND_ONOFF_DEFAULT: i32 = 257; // set_play_silent_sound_onoff_default
pub const ELM_SYSCOM_SET_LOAD_ENABLE_FLAG: i32 = 258; // set_load_enable_flag
pub const ELM_SYSCOM_GET_LOAD_ENABLE_FLAG: i32 = 259; // get_load_enable_flag
pub const ELM_SYSCOM_SET_LOAD_EXIST_FLAG: i32 = 260; // set_load_exist_flag
pub const ELM_SYSCOM_GET_LOAD_EXIST_FLAG: i32 = 261; // get_load_exist_flag
pub const ELM_SYSCOM_SET_MOV_VOLUME: i32 = 263; // set_mov_volume
pub const ELM_SYSCOM_SET_MOV_VOLUME_DEFAULT: i32 = 264; // set_mov_volume_default
pub const ELM_SYSCOM_GET_MOV_VOLUME: i32 = 265; // get_mov_volume
pub const ELM_SYSCOM_SET_MOV_ONOFF: i32 = 266; // set_mov_onoff
pub const ELM_SYSCOM_SET_MOV_ONOFF_DEFAULT: i32 = 267; // set_mov_onoff_default
pub const ELM_SYSCOM_GET_MOV_ONOFF: i32 = 268; // get_mov_onoff
pub const ELM_SYSCOM_END_LOAD: i32 = 269; // end_load
pub const ELM_SYSCOM_GET_END_SAVE_EXIST: i32 = 270; // get_end_save_exist
pub const ELM_SYSCOM_END_SAVE: i32 = 271; // end_save
pub const ELM_SYSCOM_SET_SOUND_VOLUME: i32 = 277; // set_sound_volume
pub const ELM_SYSCOM_SET_SOUND_VOLUME_DEFAULT: i32 = 278; // set_sound_volume_default
pub const ELM_SYSCOM_GET_SOUND_VOLUME: i32 = 279; // get_sound_volume
pub const ELM_SYSCOM_SET_SOUND_ONOFF: i32 = 280; // set_sound_onoff
pub const ELM_SYSCOM_SET_SOUND_ONOFF_DEFAULT: i32 = 281; // set_sound_onoff_default
pub const ELM_SYSCOM_GET_SOUND_ONOFF: i32 = 282; // get_sound_onoff
pub const ELM_SYSCOM_SET_FONT_NAME: i32 = 283; // set_font_name
pub const ELM_SYSCOM_GET_FONT_NAME: i32 = 284; // get_font_name
pub const ELM_SYSCOM_IS_FONT_EXIST: i32 = 285; // is_font_exist
pub const ELM_SYSCOM_CREATE_CAPTURE_BUFFER: i32 = 286; // create_capture_buffer
pub const ELM_SYSCOM_DESTROY_CAPTURE_BUFFER: i32 = 287; // destroy_capture_buffer
pub const ELM_SYSCOM_REPLAY_KOE: i32 = 288; // replay_koe
pub const ELM_SYSCOM_GET_REPLAY_KOE_KOE_NO: i32 = 289; // get_replay_koe_koe_no
pub const ELM_SYSCOM_CAPTURE_AND_SAVE_BUFFER_TO_PNG: i32 = 290; // capture_and_save_buffer_to_png
pub const ELM_SYSCOM_GET_REPLAY_KOE_CHARA_NO: i32 = 291; // get_replay_koe_chara_no
pub const ELM_SYSCOM_CHECK_REPLAY_KOE: i32 = 292; // check_replay_koe
pub const ELM_SYSCOM_CLEAR_REPLAY_KOE: i32 = 293; // clear_replay_koe
pub const ELM_SYSCOM_SET_FONT_BOLD: i32 = 296; // set_font_bold
pub const ELM_SYSCOM_SET_FONT_DECORATION: i32 = 297; // set_font_decoration
pub const ELM_SYSCOM_SET_FONT_BOLD_DEFAULT: i32 = 298; // set_font_bold_default
pub const ELM_SYSCOM_SET_FONT_DECORATION_DEFAULT: i32 = 299; // set_font_decoration_default
pub const ELM_SYSCOM_SET_LOCAL_EXTRA_SWITCH_ONOFF_FLAG: i32 = 300; // set_local_extra_switch_onoff_flag
pub const ELM_SYSCOM_GET_LOCAL_EXTRA_SWITCH_ONOFF_FLAG: i32 = 301; // get_local_extra_switch_onoff_flag
pub const ELM_SYSCOM_SET_LOCAL_EXTRA_SWITCH_ENABLE_FLAG: i32 = 302; // set_local_extra_switch_enable_flag
pub const ELM_SYSCOM_GET_LOCAL_EXTRA_SWITCH_ENABLE_FLAG: i32 = 303; // get_local_extra_switch_enable_flag
pub const ELM_SYSCOM_SET_LOCAL_EXTRA_SWITCH_EXIST_FLAG: i32 = 304; // set_local_extra_switch_exist_flag
pub const ELM_SYSCOM_GET_LOCAL_EXTRA_SWITCH_EXIST_FLAG: i32 = 305; // get_local_extra_switch_exist_flag
pub const ELM_SYSCOM_CHECK_LOCAL_EXTRA_SWITCH_ENABLE: i32 = 306; // check_local_extra_switch_enable
pub const ELM_SYSCOM_GET_FONT_BOLD: i32 = 307; // get_font_bold
pub const ELM_SYSCOM_GET_FONT_DECORATION: i32 = 308; // get_font_decoration
pub const ELM_SYSCOM_CHECK_WINDOW_MODE_SIZE_ENABLE: i32 = 309; // check_window_mode_size_enable
pub const ELM_SYSCOM_MSG_BACK_LOAD: i32 = 310; // msg_back_load
pub const ELM_SYSCOM_SET_MOUSE_CURSOR_HIDE_ONOFF: i32 = 311; // set_mouse_cursor_hide_onoff
pub const ELM_SYSCOM_SET_MOUSE_CURSOR_HIDE_ONOFF_DEFAULT: i32 = 312; // set_mouse_cursor_hide_onoff_default
pub const ELM_SYSCOM_GET_MOUSE_CURSOR_HIDE_ONOFF: i32 = 313; // get_mouse_cursor_hide_onoff
pub const ELM_SYSCOM_SAVE_CAPTURE_BUFFER_TO_FILE: i32 = 314; // save_capture_buffer_to_file
pub const ELM_SYSCOM_LOAD_FLAG_FROM_CAPTURE_FILE: i32 = 315; // load_flag_from_capture_file
pub const ELM_SYSCOM_CAPTURE_TO_CAPTURE_BUFFER: i32 = 316; // capture_to_capture_buffer
pub const ELM_SYSCOM_SET_MOUSE_CURSOR_HIDE_TIME: i32 = 317; // set_mouse_cursor_hide_time
pub const ELM_SYSCOM_SET_MOUSE_CURSOR_HIDE_TIME_DEFAULT: i32 = 318; // set_mouse_cursor_hide_time_default
pub const ELM_SYSCOM_GET_MOUSE_CURSOR_HIDE_TIME: i32 = 319; // get_mouse_cursor_hide_time
pub const ELM_SYSCOM_GET_SAVE_APPEND_DIR: i32 = 320; // get_save_append_dir
pub const ELM_SYSCOM_GET_SAVE_APPEND_NAME: i32 = 321; // get_save_append_name
pub const ELM_SYSCOM_GET_QUICK_SAVE_APPEND_DIR: i32 = 322; // get_quick_save_append_dir
pub const ELM_SYSCOM_GET_QUICK_SAVE_APPEND_NAME: i32 = 323; // get_quick_save_append_name
pub const ELM_SYSCOM_GET_SAVE_FULL_MESSAGE: i32 = 324; // get_save_full_message
pub const ELM_SYSCOM_GET_QUICK_SAVE_FULL_MESSAGE: i32 = 325; // get_quick_save_full_message
pub const ELM_SYSCOM_SET_FONT_NAME_DEFAULT: i32 = 326; // set_font_name_default
pub const ELM_SYSCOM_SET_RETURN_SCENE_ONCE: i32 = 328; // set_return_scene_once
pub const ELM_SYSCOM_CHECK_MSG_BACK_OPEN: i32 = 329; // check_msg_back_open
pub const ELM_SYSCOM_GET_SYSTEM_EXTRA_INT_VALUE: i32 = 330; // get_system_extra_int_value
pub const ELM_SYSCOM_GET_SYSTEM_EXTRA_STR_VALUE: i32 = 331; // get_system_extra_str_value
pub const ELM_SYSCOM__HIDE_MOUSE_CURSOR_ONESHOT: i32 = 332; // __hide_mouse_cursor_oneshot
pub const ELM_SYSCOM_CALL_CONFIG_JOYPAD_MENU: i32 = 334; // call_config_joypad_menu
pub const ELM_SYSCOM_MSG_BACK_GET_MESSAGE: i32 = 336; // msg_back_get_message
pub const ELM_SYSCOM_MSG_BACK_GET_NAME: i32 = 337; // msg_back_get_name
pub const ELM_SYSCOM_MSG_BACK_GET_KOE_NO: i32 = 338; // msg_back_get_koe_no
pub const ELM_SYSCOM_MSG_BACK_GET_CHR_NO: i32 = 339; // msg_back_get_chr_no
