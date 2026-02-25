use super::*;

impl Vm {
    fn resolve_assign_intlist_rhs(&self, rhs: &Prop) -> Result<Vec<i32>> {
        match &rhs.value {
            PropValue::IntList(v) => Ok(v.clone()),
            PropValue::Element(el) => {
                let alias = self.resolve_command_element_alias(el);
                self.resolve_intlist_source(&alias)
                    .ok_or_else(|| anyhow::anyhow!("CD_ASSIGN: unresolved INTLIST source element"))
            }
            _ => bail!("CD_ASSIGN call.L: rhs type mismatch for whole-list assign"),
        }
    }

    fn resolve_assign_strlist_rhs(&self, rhs: &Prop) -> Result<Vec<String>> {
        match &rhs.value {
            PropValue::StrList(v) => Ok(v.clone()),
            PropValue::Element(el) => {
                let alias = self.resolve_command_element_alias(el);
                self.resolve_strlist_source(&alias)
                    .ok_or_else(|| anyhow::anyhow!("CD_ASSIGN: unresolved STRLIST source element"))
            }
            _ => bail!("CD_ASSIGN call.K: rhs type mismatch for whole-list assign"),
        }
    }
}
