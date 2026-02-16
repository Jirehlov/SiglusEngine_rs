pub const ELM_OWNER_CALL_PROP: i32 = 125;
pub const ELM_OWNER_USER_CMD: i32 = 126;
pub const ELM_OWNER_USER_PROP: i32 = 127;

#[inline]
fn elm_owner(v: i32) -> u8 {
    ((v as u32) >> 24) as u8
}

pub fn is_user_prop(v: i32) -> bool {
    elm_owner(v) as i32 == crate::elm::owner::ELM_OWNER_USER_PROP
}

pub fn is_call_prop(v: i32) -> bool {
    elm_owner(v) as i32 == crate::elm::owner::ELM_OWNER_CALL_PROP
}

pub fn is_user_cmd(v: i32) -> bool {
    elm_owner(v) as i32 == crate::elm::owner::ELM_OWNER_USER_CMD
}
