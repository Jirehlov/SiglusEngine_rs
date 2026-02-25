impl Vm {
    fn object_scope_key(list_id: i32, obj_idx: i32, stage_idx: Option<i32>) -> (i32, i32, i32) {
        (stage_idx.unwrap_or(-1), list_id, obj_idx)
    }

    fn object_gan_track_clear(&mut self, list_id: i32, obj_idx: i32, stage_idx: Option<i32>) {
        let key = Self::object_scope_key(list_id, obj_idx, stage_idx);
        self.object_gan_loaded_path.remove(&key);
        self.object_gan_started_set.remove(&key);
    }

    fn object_gan_track_load_changed(
        &mut self,
        list_id: i32,
        obj_idx: i32,
        stage_idx: Option<i32>,
        gan_path: &str,
    ) -> bool {
        let key = Self::object_scope_key(list_id, obj_idx, stage_idx);
        let changed = self
            .object_gan_loaded_path
            .get(&key)
            .map(|prev| prev.as_str() != gan_path)
            .unwrap_or(true);
        self.object_gan_loaded_path
            .insert(key, gan_path.to_string());
        if changed {
            self.object_gan_started_set.remove(&key);
        }
        changed
    }

    fn object_gan_track_start_changed(
        &mut self,
        list_id: i32,
        obj_idx: i32,
        stage_idx: Option<i32>,
        set_no: i32,
    ) -> bool {
        let key = Self::object_scope_key(list_id, obj_idx, stage_idx);
        if !self.object_gan_loaded_path.contains_key(&key) {
            return false;
        }
        let changed = self
            .object_gan_started_set
            .get(&key)
            .copied()
            .map(|prev| prev != set_no)
            .unwrap_or(true);
        self.object_gan_started_set.insert(key, set_no);
        changed
    }
}
