use crate::{Opcode, RwasmExecutor};

impl<'a, T> RwasmExecutor<'a, T> {
    pub fn capture_pre_state(&mut self, instr: &Opcode) {
        let pre_state = self.pre_state(instr);
        self.store.jolt_tracer.capture_pre_state(pre_state);
    }

    pub fn capture_post_state(&mut self, instr: &Opcode) {
        let post_state = self.post_state(instr);
        self.store.jolt_tracer.capture_post_state(post_state);
    }

    // TODO: change name to sp1_sp2_pre_state
    /// # Returns
    /// - `sp`: current stack pointer
    /// - `sp1`: (sp - 1, value at sp - 1)
    /// - `sp2`: (sp - 2, value at sp - 2)
    fn pre_state(&mut self, instr: &Opcode) -> (u64, Option<(u64, u64)>, Option<(u64, u64)>) {
        match *instr {
            Opcode::I32Eq
            | Opcode::I32Add
            | Opcode::I32Mul
            | Opcode::I32And
            | Opcode::I32Or
            | Opcode::I32Xor => self.binop_pre_state(),

            Opcode::I32Store(_) => self.binop_pre_state(), /* TODO: This feels wrong, but it */
            // matches the original logic
            Opcode::I32Load(_) => {
                let (sp, sp1_state) = self.unary_op_pre_state();
                (
                    sp,
                    Some(sp1_state.unwrap()), // address
                    None,                     // sp2 is not used in store
                )
            }
            Opcode::I32Const(..) => (self.sp(), None, None),

            // HACK: These are unimplemented opcodes
            Opcode::ReturnCallInternal(_)
            | Opcode::Return
            | Opcode::StackCheck(_)
            | Opcode::SignatureCheck(_)
            | Opcode::ConsumeFuel(_) => (self.sp(), None, None),
            _ => unimplemented!(),
        }
    }

    // TODO: change name to ...
    fn post_state(&mut self, instr: &Opcode) -> Option<(u64, u64)> {
        match *instr {
            Opcode::I32Eq
            | Opcode::I32Add
            | Opcode::I32Mul
            | Opcode::I32And
            | Opcode::I32Or
            | Opcode::I32Xor => self.spd_post_state(),
            Opcode::I32Const(_) => self.spd_post_state(),

            Opcode::I32Load(_) => self.spd_post_state(),
            // Store does not write to stack
            Opcode::I32Store(_) => None,

            // HACK: These are unimplemented opcodes
            Opcode::ReturnCallInternal(_)
            | Opcode::Return
            | Opcode::StackCheck(_)
            | Opcode::SignatureCheck(_)
            | Opcode::ConsumeFuel(_) => None,
            _ => unimplemented!(),
        }
    }

    // TODO: We should not have duplicate docs
    /// # Returns
    /// - `sp`: current stack pointer
    /// - `sp1`: (sp - 1, value at sp - 1)
    /// - `sp2`: (sp - 2, value at sp - 2)
    fn binop_pre_state(&mut self) -> (u64, Option<(u64, u64)>, Option<(u64, u64)>) {
        let sp1_val = self.sp.last();
        let sp2_val = self.sp.nth_back(2);
        let sp = self.sp();
        (
            sp,
            Some((sp - 1, sp1_val.as_u64())),
            Some((sp - 2, sp2_val.as_u64())),
        )
    }

    /// # Returns
    /// - `sp`: current stack pointer
    /// - `sp1`: (sp - 1, value at sp - 1)
    fn unary_op_pre_state(&mut self) -> (u64, Option<(u64, u64)>) {
        let sp1_val = self.sp.last();
        let sp = self.sp();
        (sp, Some((sp - 1, sp1_val.as_u64())))
    }

    fn spd_post_state(&mut self) -> Option<(u64, u64)> {
        let spd_val = self.sp.last();
        let sp = self.sp();
        Some((sp - 1, spd_val.as_u64()))
    }

    fn sp(&mut self) -> u64 {
        let sp_isize = self.sp.offset_from(self.value_stack.base_ptr());
        sp_isize as u64
    }
}
