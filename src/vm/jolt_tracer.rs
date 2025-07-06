// //! Tracer library for a Jolt zkVM.

use crate::{MemoryState, SPState, StackState, WASMInstruction, WASMTraceRow};
use core::cell::RefCell;

#[derive(Debug)]
pub struct JoltTracer {
    pub rows: RefCell<Vec<WASMTraceRow>>,
    open: RefCell<bool>,
}

impl JoltTracer {
    pub fn start_instruction(&self, inst: WASMInstruction) {
        let mut inst = inst;
        inst.address = u64::from(inst.address as u32);
        *self.open.try_borrow_mut().unwrap() = true;
        self.rows.try_borrow_mut().unwrap().push(WASMTraceRow {
            instruction: inst,
            stack_state: StackState::default(),
            memory_state: None,
        });
    }

    pub fn capture_pre_state(&self, (sp, sp1, sp2): (u64, Option<SPState>, Option<SPState>)) {
        if !*self.open.try_borrow().unwrap() {
            return;
        }
        let mut rows = self.rows.try_borrow_mut().unwrap();
        let row = rows.last_mut().unwrap();
        row.stack_state.sp = sp;
        row.stack_state.sp1 = sp1;
        row.stack_state.sp2 = sp2;
    }

    pub fn capture_post_state(&self, spd: Option<SPState>) {
        if !*self.open.try_borrow().unwrap() {
            return;
        }
        let mut rows = self.rows.try_borrow_mut().unwrap();
        let row = rows.last_mut().unwrap();
        row.stack_state.spd = spd;
    }

    pub fn push_memory(&self, memory_state: MemoryState) {
        if !*self.open.try_borrow().unwrap() {
            return;
        }
        if let Some(row) = self.rows.try_borrow_mut().unwrap().last_mut() {
            row.memory_state = Some(memory_state);
        }
    }

    pub fn end_instruction(&self) {
        *self.open.try_borrow_mut().unwrap() = false;
    }
}

impl Default for JoltTracer {
    fn default() -> Self {
        Self {
            rows: RefCell::new(Vec::new()),
            open: RefCell::new(false),
        }
    }
}
