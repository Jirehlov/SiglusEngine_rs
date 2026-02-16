use std::{collections::HashMap, sync::Arc};

use anyhow::Result;

use crate::{
    dat::SceneDat,
    pck,
    vm::{Host, SceneProvider, Vm, VmOptions, VmPersistentState, VmStats},
};

/// High-level runtime wrapper around the VM + Scene.pck.
pub struct Runtime {
    pub pack: pck::Pack,
    pub scenes: HashMap<String, Arc<SceneDat>>,
}

impl Runtime {
    pub fn new(pack: pck::Pack) -> Result<Self> {
        let mut scenes: HashMap<String, Arc<SceneDat>> = HashMap::new();
        for (name, idx) in pack.scene_name_to_index.iter() {
            let dat = Arc::new(SceneDat::parse(name.clone(), pack.scenes[*idx].clone())?);
            scenes.insert(name.clone(), dat);
        }
        Ok(Self { pack, scenes })
    }

    pub fn run_scene_z(
        &mut self,
        scene: &str,
        z_label: i32,
        host: &mut dyn Host,
        max_steps: Option<u64>,
    ) -> Result<u64> {
        let (steps, _stats) =
            self.run_scene_z_with_options(scene, z_label, host, max_steps, VmOptions::default())?;
        Ok(steps)
    }

    pub fn run_scene_z_with_options(
        &mut self,
        scene: &str,
        z_label: i32,
        host: &mut dyn Host,
        max_steps: Option<u64>,
        options: VmOptions,
    ) -> Result<(u64, VmStats)> {
        let dat = self.get_scene(scene)?;
        let mut vm = Vm::new(scene.to_string(), dat);
        if let Some(m) = max_steps {
            vm.max_steps = m;
        }
        vm.set_options(options);
        vm.lexer.jump_to_z_label(z_label)?;
        vm.run(host, self)?;
        Ok((vm.steps, vm.stats.clone()))
    }

    pub fn run_scene_z_with_options_and_persistent_state(
        &mut self,
        scene: &str,
        z_label: i32,
        host: &mut dyn Host,
        max_steps: Option<u64>,
        options: VmOptions,
        initial_state: Option<&VmPersistentState>,
    ) -> Result<(u64, VmStats, VmPersistentState)> {
        let dat = self.get_scene(scene)?;
        let mut vm = Vm::new(scene.to_string(), dat);
        if let Some(m) = max_steps {
            vm.max_steps = m;
        }
        vm.set_options(options);
        if let Some(st) = initial_state {
            vm.apply_persistent_state(st);
        }
        vm.lexer.jump_to_z_label(z_label)?;
        vm.run(host, self)?;
        Ok((vm.steps, vm.stats.clone(), vm.snapshot_persistent_state()))
    }
}

impl SceneProvider for Runtime {
    fn get_scene(&mut self, scene: &str) -> Result<Arc<SceneDat>> {
        // Return cached scene if present; else decode from pack.
        if let Some(x) = self.scenes.get(scene) {
            return Ok(x.clone());
        }
        if let Some(idx) = self.pack.scene_name_to_index.get(scene) {
            let dat = Arc::new(SceneDat::parse(
                scene.to_string(),
                self.pack.scenes[*idx].clone(),
            )?);
            self.scenes.insert(scene.to_string(), dat.clone());
            return Ok(dat);
        }
        anyhow::bail!("scene not found: {}", scene);
    }

    fn inc_cmd_count(&self) -> i32 {
        self.pack.header.inc_cmd_cnt
    }

    fn get_inc_cmd_target(&mut self, user_cmd_id: i32) -> Result<Option<(String, i32)>> {
        if user_cmd_id < 0 {
            return Ok(None);
        }
        let idx = user_cmd_id as usize;
        if idx >= self.pack.inc_cmd_list.len() {
            return Ok(None);
        }
        let (scn_no, offset) = self.pack.inc_cmd_list[idx];
        if scn_no < 0 {
            return Ok(None);
        }
        let scn_idx = scn_no as usize;
        if scn_idx >= self.pack.scene_names.len() {
            return Ok(None);
        }
        let scene_name = self.pack.scene_names[scn_idx].to_string_lossy();
        Ok(Some((scene_name, offset)))
    }
}
