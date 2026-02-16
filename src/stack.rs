use anyhow::Result;

/// C++ ifc_stack equivalent: separate int/str stacks + element points.
#[derive(Debug, Default, Clone)]
pub struct IfcStack {
    pub ints: Vec<i32>,
    pub strs: Vec<String>,
    /// stack_point_list: each entry is an index into `ints` (start of element segment)
    pub points: Vec<usize>,
}

impl IfcStack {
    #[inline]
    pub fn push_int(&mut self, v: i32) {
        self.ints.push(v);
    }

    #[inline]
    pub fn pop_int(&mut self) -> Result<i32> {
        Ok(self.ints.pop().unwrap_or(0))
    }

    #[inline]
    pub fn back_int(&self) -> Result<i32> {
        Ok(self.ints.last().copied().unwrap_or(0))
    }

    #[inline]
    pub fn push_str(&mut self, s: String) {
        self.strs.push(s);
    }

    #[inline]
    pub fn pop_str(&mut self) -> Result<String> {
        Ok(self.strs.pop().unwrap_or_default())
    }

    #[inline]
    pub fn back_str(&self) -> Result<String> {
        Ok(self.strs.last().cloned().unwrap_or_default())
    }

    /// CD_ELM_POINT
    #[inline]
    pub fn elm_point(&mut self) {
        self.points.push(self.ints.len());
    }

    /// tnm_stack_pop_element
    pub fn pop_element(&mut self) -> Result<Vec<i32>> {
        let Some(start) = self.points.pop() else {
            return Ok(Vec::new());
        };
        if start > self.ints.len() {
            return Ok(Vec::new());
        }
        let elm_cnt = self.ints.len() - start;
        let mut element = Vec::with_capacity(elm_cnt);
        for _ in 0..elm_cnt {
            element.push(self.pop_int()?);
        }
        element.reverse();
        Ok(element)
    }

    /// tnm_stack_copy_element
    pub fn copy_element(&mut self) -> Result<()> {
        let Some(&start) = self.points.last() else {
            return Ok(());
        };
        if start > self.ints.len() {
            return Ok(());
        }
        let seg = self.ints[start..].to_vec();
        self.points.push(self.ints.len());
        self.ints.extend_from_slice(&seg);
        Ok(())
    }

    /// Helper for stubbing element return values.
    pub fn push_element(&mut self, element: &[i32]) {
        self.points.push(self.ints.len());
        self.ints.extend_from_slice(element);
    }
}
