#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmSyscomWaitPhase {
    ReturnToMenu,
    ReturnToSel,
    EndGame,
    EndLoadPre,
    EndLoadPost,
    ProcOther,
    NonSyscom,
}

impl VmSyscomWaitPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReturnToMenu => "return_to_menu",
            Self::ReturnToSel => "return_to_sel",
            Self::EndGame => "end_game",
            Self::EndLoadPre => "end_load_pre",
            Self::EndLoadPost => "end_load_post",
            Self::ProcOther => "proc_other",
            Self::NonSyscom => "non_syscom",
        }
    }
}

pub fn classify_syscom_wait_owner(owner_id: i32) -> VmSyscomWaitPhase {
    match owner_id {
        SYSCOM_WAIT_OWNER_PROC_RETURN_TO_MENU => VmSyscomWaitPhase::ReturnToMenu,
        SYSCOM_WAIT_OWNER_PROC_RETURN_TO_SEL => VmSyscomWaitPhase::ReturnToSel,
        SYSCOM_WAIT_OWNER_PROC_END_GAME => VmSyscomWaitPhase::EndGame,
        SYSCOM_WAIT_OWNER_END_LOAD_PRE_QUEUE => VmSyscomWaitPhase::EndLoadPre,
        SYSCOM_WAIT_OWNER_END_LOAD_POST_QUEUE => VmSyscomWaitPhase::EndLoadPost,
        id if id <= SYSCOM_WAIT_OWNER_PROC_BASE && id > SYSCOM_WAIT_OWNER_END_LOAD_PRE_QUEUE => {
            VmSyscomWaitPhase::ProcOther
        }
        _ => VmSyscomWaitPhase::NonSyscom,
    }
}

pub fn format_syscom_wait_trace(
    owner_id: i32,
    key_skip: bool,
    status: i32,
    proc_depth: i32,
    proc_top: i32,
) -> String {
    let phase = classify_syscom_wait_owner(owner_id).as_str();
    format!(
        "vm.wait owner={owner_id} phase={phase} status={status} key_skip={key_skip} depth={proc_depth} top={proc_top}"
    )
}
