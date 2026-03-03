use super::*;

impl Vm {
    pub(super) fn is_flick_scene_allowed(&self, host: &mut dyn Host) -> bool {
        if self.game_timer_move_flag == 0 {
            return false;
        }
        if self.msg_back_open_flag != 0 {
            return false;
        }
        if self.syscom_menu_disable_flag != 0 {
            return false;
        }
        let hide_mwnd_active = self.hide_mwnd_onoff_flag != 0
            && self.hide_mwnd_enable_flag != 0
            && self.hide_mwnd_exist_flag != 0
            && !self.script_hide_mwnd_disable;
        if hide_mwnd_active {
            return false;
        }
        if self.excall_allocated.iter().any(|v| *v) {
            return false;
        }
        if host.on_movie_is_playing() {
            return false;
        }
        true
    }

    ///
    /// Consumes `do_load_after_call_flag` by performing a farcall to
    /// `load_after_call_scene` / `load_after_call_z_no` from INI config.
    /// The farcall is issued with `frame_action_flag = true` so the new
    /// call frame is automatically popped on return.
    ///
    /// Must be called **after** `run_syscom_proc_queue` completes (same as
    /// C++ frame ordering: `frame_main_proc` → `frame_action_proc`).
    pub fn frame_action_proc(
        &mut self,
        host: &mut dyn Host,
        provider: &mut dyn SceneProvider,
    ) -> Result<()> {
        if self.do_load_after_call_flag != 0 {
            // Consume once per frame, matching C++ `frame_local` which resets the
            // flag to false at the start of every frame.
            self.do_load_after_call_flag = 0;

            if let Some(scene) = self.options.load_after_call_scene.clone() {
                if !scene.is_empty() {
                    let z = self.options.load_after_call_z_no;
                    host.on_frame_action_load_after_call(&scene, z);

                    // C++ calls tnm_scene_proc_farcall(scene, z, FM_VOID, false, true)
                    // which pushes a new call with frame_action_flag=true and then
                    // immediately pushes TNM_PROC_TYPE_SCRIPT → tnm_proc_script().
                    self.proc_farcall_like(
                        &scene,
                        z,
                        crate::elm::form::VOID,
                        &[],
                        false,
                        provider,
                    )?;
                    if let Some(f) = self.frames.last_mut() {
                        f.frame_action_flag = true;
                    }
                    self.push_script_proc();
                    // C++ then enters tnm_proc_script() inline; equivalent is
                    // running the VM from the new PC until the farcall returns.
                    self.run_inner(host, provider)?;
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self, host: &mut dyn Host, provider: &mut dyn SceneProvider) -> Result<()> {
        self.run_inner(host, provider).with_context(|| {
            format!(
                "vm: error at pc={} line={} scene={}",
                self.last_pc, self.last_line_no, self.last_scene
            )
        })
    }
}
