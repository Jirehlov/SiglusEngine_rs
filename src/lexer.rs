use std::sync::Arc;

use anyhow::{bail, Result};

use crate::dat::SceneDat;

/// C++-style scene lexer over .dat scn_bytes.
#[derive(Debug, Clone)]
pub struct SceneLexer {
    pub dat: Arc<SceneDat>,
    pub pc: usize,
    pub cur_line_no: i32,
}

impl SceneLexer {
    pub fn new(dat: Arc<SceneDat>) -> Self {
        Self {
            dat,
            pc: 0,
            cur_line_no: 0,
        }
    }

    /// Switch the underlying scene data (used by jump/farcall).
    ///
    /// Note: resets pc/line_no; callers should set pc afterwards.
    pub fn set_scene(&mut self, dat: Arc<SceneDat>) {
        self.dat = dat;
        self.pc = 0;
        self.cur_line_no = 0;
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.pc >= self.dat.scn_bytes.len()
    }

    #[inline]
    pub fn pop_u8(&mut self) -> Result<u8> {
        if self.pc >= self.dat.scn_bytes.len() {
            bail!("lexer: eof");
        }
        let b = self.dat.scn_bytes[self.pc];
        self.pc += 1;
        Ok(b)
    }

    #[inline]
    pub fn pop_i32(&mut self) -> Result<i32> {
        if self.pc + 4 > self.dat.scn_bytes.len() {
            bail!("lexer: truncated i32 at pc={}", self.pc);
        }
        let b = &self.dat.scn_bytes[self.pc..self.pc + 4];
        self.pc += 4;
        Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    #[inline]
    pub fn pop_str_ret(&mut self) -> Result<String> {
        let idx = self.pop_i32()?;
        Ok(self.get_string(idx))
    }

    pub fn get_string(&self, str_index: i32) -> String {
        if str_index < 0 {
            return String::new();
        }
        let i = str_index as usize;
        if i >= self.dat.strings.len() {
            return String::new();
        }
        self.dat.strings[i].to_string_lossy()
    }

    #[inline]
    fn checked_jump_pc(&self, ofs: i32, what: &str) -> Result<usize> {
        if ofs < 0 {
            bail!("lexer: negative {}", what);
        }
        let pc = ofs as usize;
        if pc > self.dat.scn_bytes.len() {
            bail!(
                "lexer: {} out of bytecode range: {} > {}",
                what,
                pc,
                self.dat.scn_bytes.len()
            );
        }
        Ok(pc)
    }

    pub fn jump_to_label(&mut self, label_no: i32) -> Result<()> {
        if label_no < 0 {
            bail!("lexer: negative label_no");
        }
        let i = label_no as usize;
        if i >= self.dat.labels.len() {
            bail!("lexer: label_no out of range: {}", label_no);
        }
        let ofs = self.dat.labels[i];
        self.pc = self.checked_jump_pc(ofs, "label_ofs")?;
        Ok(())
    }

    pub fn jump_to_z_label(&mut self, z_no: i32) -> Result<()> {
        if z_no < 0 {
            bail!("lexer: negative z_no");
        }
        let i = z_no as usize;
        if i >= self.dat.z_labels.len() {
            bail!("lexer: z_no out of range: {}", z_no);
        }
        let ofs = self.dat.z_labels[i];
        self.pc = self.checked_jump_pc(ofs, "z_label_ofs")?;
        Ok(())
    }

    pub fn jump_to_user_cmd(&mut self, user_cmd_id: i32) -> Result<()> {
        // cmd_label_list is (cmd_id, offset)
        for (cmd_id, ofs) in &self.dat.cmd_labels {
            if *cmd_id == user_cmd_id {
                self.pc =
                    self.checked_jump_pc(*ofs, &format!("user_cmd {} offset", user_cmd_id))?;
                return Ok(());
            }
        }
        bail!("lexer: user_cmd {} not found", user_cmd_id);
    }

    pub fn jump_to_scn_cmd_index(&mut self, scn_cmd_no: i32) -> Result<()> {
        if scn_cmd_no < 0 {
            bail!("lexer: negative scn_cmd_no");
        }
        let i = scn_cmd_no as usize;
        if i >= self.dat.scn_cmds.len() {
            bail!("lexer: scn_cmd_no out of range: {}", scn_cmd_no);
        }
        let ofs = self.dat.scn_cmds[i];
        self.pc = self.checked_jump_pc(ofs, &format!("scn_cmd {} offset", scn_cmd_no))?;
        Ok(())
    }
}
