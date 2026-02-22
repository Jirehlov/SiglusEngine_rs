/// STUB(C++: `cmd_object.cpp` / `elm_object.cpp`)
/// - C++ 对照：`cmd_object.cpp` 的 object-int 查询通道（`cmd_object*` -> host `on_object_query`）在未知 sub/selector
///   时返回 0，且不产生额外副作用；Rust 这里复用同一兜底策略。
/// - 当前缺口：原版未公开 selector 符号表；因此此处采用“静态 RIIR 可证明路径”先复刻 C++ 查询车道，
///   不依赖任何特定样本才能定义返回域。
/// - 预期行为：selector 0~8 读取 movie failure snapshot；selector<0 / selector>=9 / 参数超长 / 无对象 / 无失败快照 全部返回 0。
/// - 最小验证方向：
///   1) selector 0~8 逐一验证返回域与错误码约束；
///   2) selector 9 与参数超长时验证保持 0 返回（unknown/invalid selector path）；
///   3) WAIT/CHECK 轮询期间不应修改 failure snapshot（仅读路径）。
#[derive(Clone, Copy)]
enum IappMovieQuerySelector {
    /// selector=0, 返回状态码：0=ok，负值=失败（当前宿主错误域）。
    FailureStatusCode,
    /// selector=1, 返回打包计数：低16位=spawn_fail, 高16位=wait_fail。
    FailureCountersPacked,
    /// selector=2, 返回失败分类枚举（backend/io/unsupported）。
    FailureCategoryCode,
    /// selector=3, 返回是否不可恢复（0/1）。
    FailureUnrecoverableFlag,
    /// selector=4, 返回后端标识 hash。
    FailureBackendHash,
    /// selector=5, 返回细节信息 hash。
    FailureDetailHash,
    /// selector=6, 返回 spawn 失败次数（>=0）。
    FailureSpawnFailCount,
    /// selector=7, 返回 wait 失败次数（>=0）。
    FailureWaitFailCount,
    /// selector=8, 返回 exit 失败次数（>=0）。
    FailureExitFailCount,
}

impl IappMovieQuerySelector {
    fn from_i32(v: i32) -> Option<Self> {
        Some(match v {
            0 => Self::FailureStatusCode,
            1 => Self::FailureCountersPacked,
            2 => Self::FailureCategoryCode,
            3 => Self::FailureUnrecoverableFlag,
            4 => Self::FailureBackendHash,
            5 => Self::FailureDetailHash,
            6 => Self::FailureSpawnFailCount,
            7 => Self::FailureWaitFailCount,
            8 => Self::FailureExitFailCount,
            _ => return None,
        })
    }

    fn cxx_branch_hint(self) -> &'static str {
        match self {
            // C++ 侧未公开 selector 名称；这里记录 cmd_object 查询分支对应的 host 返回位槽，
            // 便于按 `cmd_object.cpp: tnm_command_proc_object -> switch(ELM_OBJECT_*) -> int push` 对照。
            Self::FailureStatusCode => "slot0_status",
            Self::FailureCountersPacked => "slot1_counters",
            Self::FailureCategoryCode => "slot2_category",
            Self::FailureUnrecoverableFlag => "slot3_unrecoverable",
            Self::FailureBackendHash => "slot4_backend_hash",
            Self::FailureDetailHash => "slot5_detail_hash",
            Self::FailureSpawnFailCount => "slot6_spawn_fail",
            Self::FailureWaitFailCount => "slot7_wait_fail",
            Self::FailureExitFailCount => "slot8_exit_fail",
        }
    }

    fn domain_ok(self, value: i32) -> bool {
        match self {
            Self::FailureStatusCode => value != 0,
            Self::FailureCountersPacked => value >= 0,
            Self::FailureCategoryCode => (1..=5).contains(&value),
            Self::FailureUnrecoverableFlag => value == 0 || value == 1,
            Self::FailureBackendHash | Self::FailureDetailHash => value >= 0,
            Self::FailureSpawnFailCount
            | Self::FailureWaitFailCount
            | Self::FailureExitFailCount => value >= 0,
        }
    }
}

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
            }
        }

        fn on_group_init(&mut self, stage_idx: i32, group_idx: i32) {
            if let Some(plane) = crate::gui::stage::stage_idx_to_plane(stage_idx) {
                self.groups
                    .insert((plane, group_idx), HostGroupState::default());
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
                st.result = -1;
                st.result_button_no = -1;
                st.active = false;
                let _ = st;
                self.play_cancel_se(se_no);
                return Some(-1);
            }

            match self.selection_rx.try_recv() {
                Ok(selected) => {
                    let mut decided = selected;
                    let mut cancel_se_no = -1;
                    if selected < 0 && st.cancel_enabled {
                        decided = -1;
                        cancel_se_no = st.cancel_se_no;
                    }
                    st.hit_button_no = decided;
                    st.pushed_button_no = decided;
                    st.decided_button_no = decided;
                    st.result = decided;
                    st.result_button_no = decided;
                    st.active = false;
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
                    i32::from(self.movie_playing_objects.contains(&(plane, obj_index)))
                }
                x if x == siglus::elm::objectlist::ELM_OBJECT_DISP => i32::from(st.visible),
                x if x == siglus::elm::objectlist::ELM_OBJECT_X => st.x as i32,
                x if x == siglus::elm::objectlist::ELM_OBJECT_Y => st.y as i32,
                x if x == siglus::elm::objectlist::ELM_OBJECT_PATNO => st.pat_no as i32,
                x if x == siglus::elm::objectlist::ELM_OBJECT_ORDER => st.order,
                x if x == siglus::elm::objectlist::ELM_OBJECT_LAYER => st.layer,
                x if x == siglus::elm::objectlist::ELM_OBJECT_EXIST_TYPE => 1,
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
            let Some(info) = self.movie_last_failure.get(&(plane, obj_index)) else {
                return 0;
            };
            let Some(selector) = IappMovieQuerySelector::from_i32(selector_raw) else {
                // 错误域：未知 selector 不抛错，直接返回 0。
                return 0;
            };
            let value = match selector {
                IappMovieQuerySelector::FailureStatusCode => info.status_code(),
                IappMovieQuerySelector::FailureCountersPacked => info.counters_packed(),
                IappMovieQuerySelector::FailureCategoryCode => info.category_code(),
                IappMovieQuerySelector::FailureUnrecoverableFlag => info.unrecoverable_flag(),
                IappMovieQuerySelector::FailureBackendHash => info.backend_hash(),
                IappMovieQuerySelector::FailureDetailHash => info.detail_hash(),
                IappMovieQuerySelector::FailureSpawnFailCount => info.spawn_fail_count(),
                IappMovieQuerySelector::FailureWaitFailCount => info.wait_fail_count(),
                IappMovieQuerySelector::FailureExitFailCount => info.exit_fail_count(),
            };
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
