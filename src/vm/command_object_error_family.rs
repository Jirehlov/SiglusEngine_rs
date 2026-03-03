impl Vm {
    fn object_invalid_message_for_sub(sub: i32) -> &'static str {
        use crate::elm::objectlist::*;
        if matches!(
            sub,
            ELM_OBJECT_SET_STRING
                | ELM_OBJECT_GET_STRING
                | ELM_OBJECT_SET_STRING_PARAM
                | ELM_OBJECT_CREATE_STRING
        ) {
            return "無効なコマンドが指定されました。(object.string)";
        }
        if matches!(
            sub,
            ELM_OBJECT_SET_NUMBER
                | ELM_OBJECT_GET_NUMBER
                | ELM_OBJECT_SET_NUMBER_PARAM
                | ELM_OBJECT_CREATE_NUMBER
        ) {
            return "無効なコマンドが指定されました。(object.number)";
        }
        if matches!(
            sub,
            ELM_OBJECT_CREATE_MOVIE
                | ELM_OBJECT_CREATE_MOVIE_LOOP
                | ELM_OBJECT_CREATE_MOVIE_WAIT
                | ELM_OBJECT_CREATE_MOVIE_WAIT_KEY
                | ELM_OBJECT_PAUSE_MOVIE
                | ELM_OBJECT_RESUME_MOVIE
                | ELM_OBJECT_SEEK_MOVIE
                | ELM_OBJECT_GET_MOVIE_SEEK_TIME
                | ELM_OBJECT_CHECK_MOVIE
                | ELM_OBJECT_WAIT_MOVIE
                | ELM_OBJECT_WAIT_MOVIE_KEY
                | ELM_OBJECT_END_MOVIE_LOOP
                | ELM_OBJECT_SET_MOVIE_AUTO_FREE
        ) {
            return "無効なコマンドが指定されました。(object.movie)";
        }
        if matches!(
            sub,
            ELM_OBJECT_CREATE_EMOTE
                | ELM_OBJECT_EMOTE_PLAY_TIMELINE
                | ELM_OBJECT_EMOTE_STOP_TIMELINE
                | ELM_OBJECT_EMOTE_CHECK_PLAYING
                | ELM_OBJECT_EMOTE_WAIT_PLAYING
                | ELM_OBJECT_EMOTE_WAIT_PLAYING_KEY
                | ELM_OBJECT_EMOTE_SKIP
                | ELM_OBJECT_EMOTE_PASS
                | ELM_OBJECT_EMOTE_KOE_CHARA_NO
                | ELM_OBJECT_EMOTE_MOUTH_VOLUME
        ) {
            return "無効なコマンドが指定されました。(object.emote)";
        }
        if matches!(
            sub,
            ELM_OBJECT_CLEAR_BUTTON
                | ELM_OBJECT_SET_BUTTON
                | ELM_OBJECT_SET_BUTTON_GROUP
                | ELM_OBJECT_SET_BUTTON_PUSHKEEP
                | ELM_OBJECT_SET_BUTTON_ALPHA_TEST
                | ELM_OBJECT_SET_BUTTON_STATE_NORMAL
                | ELM_OBJECT_SET_BUTTON_STATE_SELECT
                | ELM_OBJECT_SET_BUTTON_STATE_DISABLE
                | ELM_OBJECT_SET_BUTTON_CALL
                | ELM_OBJECT_CLEAR_BUTTON_CALL
                | ELM_OBJECT_GET_BUTTON_STATE
                | ELM_OBJECT_GET_BUTTON_HIT_STATE
                | ELM_OBJECT_GET_BUTTON_REAL_STATE
                | ELM_OBJECT_GET_BUTTON_PUSHKEEP
                | ELM_OBJECT_GET_BUTTON_ALPHA_TEST
                | ELM_OBJECT_GET_BUTTON_NO
                | ELM_OBJECT_GET_BUTTON_GROUP_NO
                | ELM_OBJECT_GET_BUTTON_ACTION_NO
                | ELM_OBJECT_GET_BUTTON_SE_NO
        ) {
            return "無効なコマンドが指定されました。(object.button)";
        }
        if matches!(
            sub,
            ELM_OBJECT_SET_WEATHER_PARAM_TYPE_A | ELM_OBJECT_SET_WEATHER_PARAM_TYPE_B
        ) {
            return "無効なコマンドが指定されました。(object.weather)";
        }
        if matches!(
            sub,
            ELM_OBJECT_CREATE
                | ELM_OBJECT_CREATE_RECT
                | ELM_OBJECT_CREATE_WEATHER
                | ELM_OBJECT_CREATE_MESH
                | ELM_OBJECT_CREATE_BILLBOARD
                | ELM_OBJECT_CREATE_SAVE_THUMB
                | ELM_OBJECT_CREATE_CAPTURE_THUMB
                | ELM_OBJECT_CREATE_CAPTURE
                | ELM_OBJECT_CREATE_COPY_FROM
                | ELM_OBJECT_CREATE_FROM_CAPTURE_FILE
        ) {
            return "無効なコマンドが指定されました。(object.create)";
        }
        "無効なコマンドが指定されました。(object)"
    }

    fn object_validate_named_arg_ids(
        sub: i32,
        args: &[Prop],
        allowed: &[i32],
        host: &mut dyn Host,
    ) -> bool {
        for arg in args {
            if arg.id >= 0 && !allowed.contains(&arg.id) {
                host.on_error_fatal(Self::object_invalid_message_for_sub(sub));
                return false;
            }
        }
        true
    }

    fn object_validate_arg_range(
        sub: i32,
        args: &[Prop],
        min: usize,
        max: usize,
        host: &mut dyn Host,
    ) -> bool {
        if args.len() < min || args.len() > max {
            host.on_error_fatal(Self::object_invalid_message_for_sub(sub));
            return false;
        }
        true
    }
}
