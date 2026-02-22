macro_rules! impl_host_quake_methods {
    () => {
        fn on_quake_start(&mut self, req: siglus::vm::VmQuakeRequest) {
            let loops = req.cnt.max(0).saturating_add(1) as u64;
            let ms = (req.time_ms.max(1) as u64).saturating_mul(loops);
            self.quake_active_until = Some(Instant::now() + std::time::Duration::from_millis(ms));
            self.last_quake_request = Some(req);
            let _ = self.event_tx.send(HostEvent::StartQuake {
                req,
                started_at: Instant::now(),
            });
        }

        fn on_quake_end(&mut self) {
            self.quake_active_until = None;
            let _ = self.event_tx.send(HostEvent::EndQuake);
        }

        fn on_quake_is_active(&mut self) -> bool {
            match self.quake_active_until {
                Some(deadline) if deadline > Instant::now() => true,
                Some(_) => {
                    self.quake_active_until = None;
                    let _ = self.event_tx.send(HostEvent::EndQuake);
                    false
                }
                None => false,
            }
        }
    };
}
