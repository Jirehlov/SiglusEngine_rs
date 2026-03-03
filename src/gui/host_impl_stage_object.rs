include!("host_impl_stage_object_iapp.rs");

macro_rules! impl_host_stage_object_methods {
    () => {
        fn on_stage_list_get_size(&mut self) -> i32 {
            3
        }

        fn on_group_alloc(&mut self, stage_idx: i32, count: i32) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                let count = count.max(0);
                self.stage_group_sizes.insert(plane, count);
                for idx in 0..count {
                    self.groups.entry((plane, idx)).or_default();
                }
                self.groups
                    .retain(|(p, idx), _| *p != plane || *idx < count);
            }
        }

        fn on_group_free(&mut self, stage_idx: i32) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                self.stage_group_sizes.insert(plane, 0);
                self.groups.retain(|(p, _), _| *p != plane);
            }
        }

        fn on_group_list_get_size(&mut self, stage_idx: i32) -> i32 {
            crate::gui::stage::stage_idx_to_plane(stage_idx)
                .and_then(|plane| self.stage_group_sizes.get(&plane).copied())
                .unwrap_or(-1)
        }

        fn on_group_sel(&mut self, stage_idx: i32, group_idx: i32, _sub: i32) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                let st = self.groups.entry((plane, group_idx)).or_default();
                st.active = true;
                st.result = -1;
                st.result_button_no = -1;
                st.decided_button_no = -1;
                st.hover_button_no = -1;
                st.press_keep_button_no = -1;
            }
        }

        fn on_group_init(&mut self, stage_idx: i32, group_idx: i32) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                let mut st = HostGroupState::default();
                st.result = -1;
                st.result_button_no = -1;
                st.hit_button_no = -1;
                st.pushed_button_no = -1;
                st.decided_button_no = -1;
                st.on_hit_no = -1;
                st.on_pushed_no = -1;
                st.on_decided_no = -1;
                st.hover_button_no = -1;
                st.press_keep_button_no = -1;
                self.groups.insert((plane, group_idx), st);
            }
        }

        fn on_group_start(&mut self, stage_idx: i32, group_idx: i32, _sub: i32) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                let st = self.groups.entry((plane, group_idx)).or_default();
                st.active = true;
                st.result = -1;
                st.result_button_no = -1;
            }
        }

        fn on_group_set_cancel(
            &mut self,
            stage_idx: i32,
            group_idx: i32,
            enabled: bool,
            se_no: i32,
        ) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                let st = self.groups.entry((plane, group_idx)).or_default();
                st.cancel_enabled = enabled;
                st.cancel_se_no = se_no;
            }
        }


        fn on_group_on_hit_no(&mut self, stage_idx: i32, group_idx: i32, button_no: i32) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                let st = self.groups.entry((plane, group_idx)).or_default();
                st.on_hit_no = button_no;
                if std::env::var("SIGLUS_GROUP_WAIT_TRACE").map(|v| v != "0").unwrap_or(false) {
                    debug!("vm.group_wait.route stage={} group={} route=on_hit no={}", stage_idx, group_idx, button_no);
                }
            }
        }

        fn on_group_on_pushed_no(&mut self, stage_idx: i32, group_idx: i32, button_no: i32) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                let st = self.groups.entry((plane, group_idx)).or_default();
                st.on_pushed_no = button_no;
                if std::env::var("SIGLUS_GROUP_WAIT_TRACE").map(|v| v != "0").unwrap_or(false) {
                    debug!("vm.group_wait.route stage={} group={} route=on_pushed no={}", stage_idx, group_idx, button_no);
                }
            }
        }

        fn on_group_on_decided_no(&mut self, stage_idx: i32, group_idx: i32, button_no: i32) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                let st = self.groups.entry((plane, group_idx)).or_default();
                st.on_decided_no = button_no;
                if std::env::var("SIGLUS_GROUP_WAIT_TRACE").map(|v| v != "0").unwrap_or(false) {
                    debug!("vm.group_wait.route stage={} group={} route=on_decided no={}", stage_idx, group_idx, button_no);
                }
            }
        }
        fn on_group_end(&mut self, stage_idx: i32, group_idx: i32) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                if let Some(st) = self.groups.get_mut(&(plane, group_idx)) {
                    st.active = false;
                }
            }
        }

        fn on_group_wait_result(&mut self, stage_idx: i32, group_idx: i32) -> Option<i32> {
            let plane = crate::gui::stage::stage_idx_to_plane(stage_idx)?;
            let current_priority = self
                .groups
                .get(&(plane, group_idx))
                .map(|s| s.cancel_priority)
                .unwrap_or(i32::MIN);
            if self.groups.iter().any(|((p, idx), st)| {
                *p == plane
                    && *idx != group_idx
                    && st.active
                    && st.cancel_priority > current_priority
            }) {
                return None;
            }

            let cancel_input = self
                .input_state
                .lock()
                .ok()
                .map(|mut state| state.cancel.use_down_up_stock())
                .unwrap_or(false);
            let trace_group_wait = std::env::var("SIGLUS_GROUP_WAIT_TRACE")
                .map(|v| v != "0")
                .unwrap_or(false);

            let st = self.groups.get_mut(&(plane, group_idx))?;
            if self.shutdown.load(Ordering::Relaxed) {
                st.result = -1;
                st.result_button_no = -1;
                st.active = false;
                return Some(-1);
            }
            if !st.active || st.result >= 0 {
                return if st.active { None } else { Some(st.result) };
            }

            if st.cancel_enabled && cancel_input {
                let se_no = st.cancel_se_no;
                st.hit_button_no = -1;
                st.pushed_button_no = -1;
                st.decided_button_no = -1;
                st.hover_button_no = -1;
                st.press_keep_button_no = -1;
                st.result = -1;
                st.result_button_no = -1;
                st.active = false;
                let _ = st;
                self.play_cancel_se(se_no);
                return Some(-1);
            }

            let mut direct_decided = None;
            if let Ok(mut state) = self.input_state.lock() {
                let cursor = (state.mouse_x as f32, state.mouse_y as f32);
                let mut object_hit_button = self.group_hit_candidate_button(plane, group_idx, cursor);

                // push_keep: when press started on one button, keep it during hold.
                if state.mouse_left.is_down {
                    if st.press_keep_button_no >= 0 {
                        object_hit_button = Some(st.press_keep_button_no);
                    }
                } else if !state.mouse_left.is_down {
                    st.press_keep_button_no = -1;
                }

                if let Some(hit) = object_hit_button {
                    st.hover_button_no = hit;
                    st.hit_button_no = hit;
                    if state.mouse_left.is_down {
                        st.pushed_button_no = hit;
                        let keep = self
                            .objects
                            .iter()
                            .find_map(|((p, idx), _)| {
                                if *p != plane {
                                    return None;
                                }
                                let btn = self.get_object_button_state(*p, *idx);
                                if btn.group_no == group_idx && btn.button_no == hit {
                                    Some(btn.push_keep != 0)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(false);
                        st.press_keep_button_no = if keep { hit } else { -1 };
                    }
                } else {
                    st.hover_button_no = -1;
                    if st.on_hit_no >= 0 && state.mouse_left.has_down_up_stock() {
                        st.hit_button_no = st.on_hit_no;
                    }
                    if st.on_pushed_no >= 0 && state.mouse_left.use_down_up_stock() {
                        st.pushed_button_no = st.on_pushed_no;
                    }
                }
                let decide_stock_before = state.decide.has_down_up_stock();
                let mouse_stock_before = state.mouse_left.has_down_up_stock();
                if let Some(hit) = object_hit_button.filter(|_| state.decide.use_down_up_stock()) {
                    direct_decided = Some(hit);
                } else if st.on_decided_no >= 0 && state.decide.use_down_up_stock() {
                    direct_decided = Some(st.on_decided_no);
                }
                if trace_group_wait {
                    debug!(
                        "vm.group_wait.stock stage={} group={} hit={:?} on_hit={} on_pushed={} on_decided={} mouse_stock={} decide_stock={} cancel_input={} direct_decided={:?}",
                        stage_idx,
                        group_idx,
                        object_hit_button,
                        st.on_hit_no,
                        st.on_pushed_no,
                        st.on_decided_no,
                        i32::from(mouse_stock_before),
                        i32::from(decide_stock_before),
                        i32::from(cancel_input),
                        direct_decided,
                    );
                }
            }
            if let Some(decided) = direct_decided {
                st.decided_button_no = decided;
                st.result = decided;
                st.result_button_no = decided;
                st.active = false;
                return Some(decided);
            }

            match self.selection_rx.try_recv() {
                Ok(selected) => {
                    let mut decided = selected;
                    let mut cancel_se_no = -1;
                    if selected < 0 && st.cancel_enabled {
                        decided = -1;
                        cancel_se_no = st.cancel_se_no;
                    }

                    if st.on_hit_no >= 0 && decided >= 0 && decided != st.on_hit_no {
                        return None;
                    }
                    st.hit_button_no = decided;

                    if st.on_pushed_no >= 0 && decided >= 0 && decided != st.on_pushed_no {
                        return None;
                    }
                    st.pushed_button_no = decided;

                    if st.on_decided_no >= 0 && decided >= 0 && decided != st.on_decided_no {
                        return None;
                    }
                    st.decided_button_no = decided;
                    st.result = decided;
                    st.result_button_no = decided;
                    st.active = false;
                    if trace_group_wait {
                        debug!(
                            "vm.group_wait.result stage={} group={} decided={} cancel_se_no={} route_hit={} route_pushed={} route_decided={}",
                            stage_idx,
                            group_idx,
                            decided,
                            cancel_se_no,
                            st.on_hit_no,
                            st.on_pushed_no,
                            st.on_decided_no
                        );
                    }
                    let _ = st;
                    if cancel_se_no >= 0 {
                        self.play_cancel_se(cancel_se_no);
                    }
                    Some(decided)
                }
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    st.result = -1;
                    st.result_button_no = -1;
                    st.hover_button_no = -1;
                    st.press_keep_button_no = -1;
                    st.active = false;
                    Some(-1)
                }
            }
        }

        fn on_group_get(&mut self, stage_idx: i32, group_idx: i32, query_id: i32) -> i32 {
            let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) else {
                return -1;
            };
            if let Some(st) = self.groups.get_mut(&(plane, group_idx)) {
                return match query_id {
                    x if x == siglus::elm::group::ELM_GROUP_GET_HIT_NO => st.hit_button_no,
                    x if x == siglus::elm::group::ELM_GROUP_GET_PUSHED_NO => st.pushed_button_no,
                    x if x == siglus::elm::group::ELM_GROUP_GET_DECIDED_NO => st.decided_button_no,
                    x if x == siglus::elm::group::ELM_GROUP_GET_RESULT => st.result,
                    x if x == siglus::elm::group::ELM_GROUP_GET_RESULT_BUTTON_NO => {
                        st.result_button_no
                    }
                    x if x == siglus::elm::group::ELM_GROUP_ORDER => st.order,
                    x if x == siglus::elm::group::ELM_GROUP_LAYER => st.layer,
                    x if x == siglus::elm::group::ELM_GROUP_CANCEL_PRIORITY => st.cancel_priority,
                    _ => -1,
                };
            }
            -1
        }

        fn on_group_property(
            &mut self,
            stage_idx: i32,
            group_idx: i32,
            property_id: i32,
            value: i32,
        ) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                let st = self.groups.entry((plane, group_idx)).or_default();
                match property_id {
                    x if x == siglus::elm::group::ELM_GROUP_ORDER => st.order = value,
                    x if x == siglus::elm::group::ELM_GROUP_LAYER => st.layer = value,
                    x if x == siglus::elm::group::ELM_GROUP_CANCEL_PRIORITY => {
                        st.cancel_priority = value
                    }
                    _ => {}
                }
            }
        }

        fn on_object_list_get_size(&mut self, list_id: i32, stage_idx: Option<i32>) -> i32 {
            if list_id != siglus::elm::objectlist::ELM_STAGE_OBJECT {
                return -1;
            }
            let Some(plane) = stage_idx.and_then(crate::gui::stage::stage_idx_to_plane) else {
                return -1;
            };
            self.stage_object_sizes
                .get(&plane)
                .copied()
                .unwrap_or_else(|| {
                    self.objects
                        .keys()
                        .filter(|(p, _)| *p == plane)
                        .map(|(_, idx)| *idx + 1)
                        .max()
                        .unwrap_or(0)
                })
        }

        fn on_object_is_use(
            &mut self,
            list_id: i32,
            obj_index: i32,
            stage_idx: Option<i32>,
        ) -> bool {
            if list_id != siglus::elm::objectlist::ELM_STAGE_OBJECT {
                return true;
            }
            if obj_index < 0 {
                return false;
            }
            let Some(plane) = stage_idx.and_then(crate::gui::stage::stage_idx_to_plane) else {
                return true;
            };
            let size = self.stage_object_sizes.get(&plane).copied().unwrap_or(0);
            obj_index < size
        }

        fn on_object_action(
            &mut self,
            list_id: i32,
            obj_index: i32,
            sub_id: i32,
            args: &[siglus::vm::Prop],
            stage_idx: Option<i32>,
        ) {
            if list_id != siglus::elm::objectlist::ELM_STAGE_OBJECT {
                return;
            }
            let Some(plane) = stage_idx.and_then(crate::gui::stage::stage_idx_to_plane) else {
                return;
            };
            if obj_index < 0 {
                if sub_id == siglus::elm::objectlist::ELM_OBJECTLIST_RESIZE {
                    let requested = args.first().and_then(|p| p.as_int()).unwrap_or(0).max(0);
                    self.stage_object_sizes.insert(plane, requested);
                }
                return;
            }
            self.apply_object_command(plane, obj_index, sub_id, args);
        }

        fn on_object_property(
            &mut self,
            list_id: i32,
            obj_index: i32,
            property_id: i32,
            value: i32,
            stage_idx: Option<i32>,
        ) {
            if list_id != siglus::elm::objectlist::ELM_STAGE_OBJECT {
                return;
            }
            let Some(plane) = stage_idx.and_then(crate::gui::stage::stage_idx_to_plane) else {
                return;
            };
            let p = siglus::vm::Prop {
                id: 0,
                form: siglus::elm::form::INT,
                value: siglus::vm::PropValue::Int(value),
            };
            self.apply_object_command(plane, obj_index, property_id, &[p]);
        }

        fn on_object_get_str(
            &mut self,
            list_id: i32,
            obj_index: i32,
            sub_id: i32,
            stage_idx: Option<i32>,
        ) -> String {
            self.refresh_movie_lifecycle();
            if list_id != siglus::elm::objectlist::ELM_STAGE_OBJECT {
                return String::new();
            }
            let Some(plane) = stage_idx.and_then(crate::gui::stage::stage_idx_to_plane) else {
                return String::new();
            };
            match sub_id {
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_FILE_NAME => self
                    .objects
                    .get(&(plane, obj_index))
                    .map(|st| st.file_name.clone())
                    .unwrap_or_default(),
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_STRING => {
                    self.get_object_string_state(plane, obj_index)
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_ELEMENT_NAME => {
                    let stage_name = match plane {
                        StagePlane::Back => "back",
                        StagePlane::Front => "front",
                        StagePlane::Next => "next",
                    };
                    format!("global.{stage_name}.object[{obj_index}]")
                }
                _ => String::new(),
            }
        }

        fn on_object_get(
            &mut self,
            list_id: i32,
            obj_index: i32,
            sub_id: i32,
            stage_idx: Option<i32>,
        ) -> i32 {
            self.refresh_movie_lifecycle();
            if list_id != siglus::elm::objectlist::ELM_STAGE_OBJECT {
                return 0;
            }
            let Some(plane) = stage_idx.and_then(crate::gui::stage::stage_idx_to_plane) else {
                return 0;
            };
            let Some(st) = self.objects.get(&(plane, obj_index)) else {
                return 0;
            };
            match sub_id {
                x if x == siglus::elm::objectlist::ELM_OBJECT_CHECK_MOVIE => {
                    let state = self.object_movie_wait_state(plane, obj_index);
                    let value = match state {
                        MovieWaitState::Pending => 1,
                        MovieWaitState::Ready => 0,
                        MovieWaitState::Failed => self.object_check_movie_failed_code(plane, obj_index),
                        MovieWaitState::Interrupted => -2,
                    };
                    if std::env::var("SIGLUS_MOVIE_WAIT_TRACE").map(|v| v != "0").unwrap_or(false) {
                        let ready_only = self
                            .objects
                            .get(&(plane, obj_index))
                            .map(|st| st.movie_ready_only)
                            .unwrap_or(false);
                        let generation_live = self.movie_generations.contains_key(&(plane, obj_index));
                        let failed_live = self.movie_last_failure.contains_key(&(plane, obj_index));
                        let interrupted_live = self.movie_interrupted_objects.contains(&(plane, obj_index));
                        log::debug!(
                            "vm.movie_wait_trace check_movie stage={:?} index={} state={:?} value={} ready_only={} generation_live={} failed_live={} interrupted_live={}",
                            plane,
                            obj_index,
                            state,
                            value,
                            ready_only,
                            generation_live,
                            failed_live,
                            interrupted_live
                        );
                        log::debug!(
                            "vm.failed_code_trace check stage={:?} index={} state={:?} value={}",
                            plane,
                            obj_index,
                            state,
                            value
                        );
                    }
                    value
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_DISP => i32::from(st.visible),
                x if x == siglus::elm::objectlist::ELM_OBJECT_X => st.x as i32,
                x if x == siglus::elm::objectlist::ELM_OBJECT_Y => st.y as i32,
                x if x == siglus::elm::objectlist::ELM_OBJECT_PATNO => st.pat_no as i32,
                x if x == siglus::elm::objectlist::ELM_OBJECT_ORDER => st.order,
                x if x == siglus::elm::objectlist::ELM_OBJECT_LAYER => st.layer,
                x if x == siglus::elm::objectlist::ELM_OBJECT_EXIST_TYPE => 1,
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_NUMBER => {
                    self.get_object_number_state(plane, obj_index)
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_MOVIE_SEEK_TIME => {
                    self.get_object_movie_seek_state(plane, obj_index)
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_BUTTON_STATE => {
                    self.get_object_button_state(plane, obj_index).state
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_BUTTON_HIT_STATE => {
                    self.get_object_button_state(plane, obj_index).hit_state
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_BUTTON_REAL_STATE => {
                    self.get_object_button_state(plane, obj_index).real_state
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_BUTTON_PUSHKEEP => {
                    self.get_object_button_state(plane, obj_index).push_keep
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_BUTTON_ALPHA_TEST => {
                    self.get_object_button_state(plane, obj_index).alpha_test
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_BUTTON_NO => {
                    self.get_object_button_state(plane, obj_index).button_no
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_BUTTON_GROUP_NO => {
                    self.get_object_button_state(plane, obj_index).group_no
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_BUTTON_ACTION_NO => {
                    self.get_object_button_state(plane, obj_index).action_no
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_GET_BUTTON_SE_NO => {
                    self.get_object_button_state(plane, obj_index).se_no
                }
                _ => 0,
            }
        }

        fn on_object_query(
            &mut self,
            list_id: i32,
            obj_index: i32,
            sub_id: i32,
            args: &[siglus::vm::Prop],
            stage_idx: Option<i32>,
        ) -> i32 {
            // C++ 路由证据链（分支级）：
            // 1) `cmd_object.cpp::tnm_command_proc_object` 仅在 object-list + object 实例可用时进入 switch；
            //    否则直接落入空路径（脚本观察到 int 0 域）。
            // 2) `switch(elm_begin[0])` 里未知/不支持分支最终进入 fatal；Rust 在 host 查询车道保持“返回 0、无副作用”，
            //    与 `__IAPP_DUMMY` 在 C++ 实机中的查询容错域一致。
            // 3) 这里继续沿用 cmd_object 的“只消费脚本声明参数个数”模型：超长参数直接回落 0。
            self.refresh_movie_lifecycle();
            if list_id != siglus::elm::objectlist::ELM_STAGE_OBJECT {
                return 0;
            }
            let Some(plane) = stage_idx.and_then(crate::gui::stage::stage_idx_to_plane) else {
                return 0;
            };
            if sub_id != siglus::elm::objectlist::ELM_OBJECT__IAPP_DUMMY {
                return 0;
            }
            if !self.objects.contains_key(&(plane, obj_index)) {
                return 0;
            }
            // 入参约束：
            // - C++ 脚本签名最多允许 10 个 int；超长参数组按错误域返回 0。
            // - 当前语义仅消费第一个 int 作为 selector。
            // - 缺省/非 int 按 0 处理，保持与 C++“参数缺失回落默认值”一致。
            if args.len() > 10 {
                return 0;
            }
            let selector_raw = args.first().and_then(|v| v.as_int()).unwrap_or(0);
            let Some(selector) = IappMovieQuerySelector::from_i32(selector_raw) else {
                // 错误域：未知 selector 不抛错，直接返回 0。
                return 0;
            };
            let value = match selector {
                IappMovieQuerySelector::FailureStatusCode
                | IappMovieQuerySelector::FailureCountersPacked
                | IappMovieQuerySelector::FailureCategoryCode
                | IappMovieQuerySelector::FailureUnrecoverableFlag
                | IappMovieQuerySelector::FailureBackendHash
                | IappMovieQuerySelector::FailureDetailHash
                | IappMovieQuerySelector::FailureSpawnFailCount
                | IappMovieQuerySelector::FailureWaitFailCount
                | IappMovieQuerySelector::FailureExitFailCount => {
                    let Some(info) = self.movie_last_failure.get(&(plane, obj_index)) else {
                        return 0;
                    };
                    match selector {
                        IappMovieQuerySelector::FailureStatusCode => info.status_code(),
                        IappMovieQuerySelector::FailureCountersPacked => info.counters_packed(),
                        IappMovieQuerySelector::FailureCategoryCode => info.category_code(),
                        IappMovieQuerySelector::FailureUnrecoverableFlag => info.unrecoverable_flag(),
                        IappMovieQuerySelector::FailureBackendHash => info.backend_hash(),
                        IappMovieQuerySelector::FailureDetailHash => info.detail_hash(),
                        IappMovieQuerySelector::FailureSpawnFailCount => info.spawn_fail_count(),
                        IappMovieQuerySelector::FailureWaitFailCount => info.wait_fail_count(),
                        IappMovieQuerySelector::FailureExitFailCount => info.exit_fail_count(),
                        _ => 0,
                    }
                }
                IappMovieQuerySelector::MovieAutoInitFlag => {
                    i32::from(self.objects.get(&(plane, obj_index)).map(|st| st.movie_auto_init).unwrap_or(true))
                }
                IappMovieQuerySelector::MovieRealTimeFlag => {
                    i32::from(self.objects.get(&(plane, obj_index)).map(|st| st.movie_real_time).unwrap_or(true))
                }
                IappMovieQuerySelector::MovieReadyOnlyFlag => {
                    i32::from(self.objects.get(&(plane, obj_index)).map(|st| st.movie_ready_only).unwrap_or(false))
                }
                IappMovieQuerySelector::EmoteRepX => self.objects.get(&(plane, obj_index)).map(|st| st.emote_rep_x).unwrap_or(0),
                IappMovieQuerySelector::EmoteRepY => self.objects.get(&(plane, obj_index)).map(|st| st.emote_rep_y).unwrap_or(0),
                IappMovieQuerySelector::CheckMovieFailedCode => self.object_check_movie_failed_code(plane, obj_index),
            };
            if std::env::var("SIGLUS_MOVIE_WAIT_TRACE").map(|v| v != "0").unwrap_or(false)
                && matches!(selector, IappMovieQuerySelector::CheckMovieFailedCode)
            {
                log::debug!(
                    "vm.failed_code_trace query stage={:?} index={} selector={} value={}",
                    plane,
                    obj_index,
                    selector_raw,
                    value
                );
            }
            if std::env::var_os("SIGLUS_DEV_ASSERT_IAPP_SELECTOR_DOMAIN").is_some()
                && !selector.domain_ok(value)
            {
                log::warn!(
                    "[dev-assert] iapp selector domain drift: selector={}({}), value={}, stage={:?}, obj={}",
                    selector_raw,
                    selector.cxx_branch_hint(),
                    value,
                    plane,
                    obj_index
                );
            }
            value
        }
    };
}
