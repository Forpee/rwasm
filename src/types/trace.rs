use crate::{Opcode, MEMORY_OPS_PER_INSTRUCTION};
use serde::{Deserialize, Serialize};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

/// A step in the execution trace. A step in the execution trace documents all the variables used at
/// a single CPU cycle, for example some variables could be [pc, sp1_read_addr, sp1_read_value,
/// RAM_READ_addr,RAM_READ_value, RAM_WRITE_addr, RAM_WRITE_value]. We also trace the
/// [`WASMInstruction`] invoked as CPU state is determinisically derived from the instr
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WASMTraceRow {
    pub instruction: WASMInstruction,
    pub stack_state: StackState,
    pub memory_state: Option<MemoryState>,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum MemoryOp {
    Read(u64),       // (address)
    Write(u64, u64), // (address, new_value)
}

impl MemoryOp {
    pub fn noop_read() -> Self {
        Self::Read(0)
    }

    pub fn noop_write() -> Self {
        Self::Write(0, 0)
    }
}

impl From<&WASMTraceRow> for [MemoryOp; MEMORY_OPS_PER_INSTRUCTION] {
    fn from(val: &WASMTraceRow) -> Self {
        let sp1_read = || MemoryOp::Read(val.stack_state.sp1.unwrap().address);
        let sp2_read = || MemoryOp::Read(val.stack_state.sp2.unwrap().address);
        let spd_write = || {
            MemoryOp::Write(
                val.stack_state.spd.unwrap().address,
                val.stack_state.spd.unwrap().value,
            )
        };

        let sp1_offset = || -> u64 {
            let sp1_val = val.stack_state.sp1.unwrap().value;
            let imm = val.instruction.imm.unwrap();
            imm.checked_add(sp1_val).expect("Memory offset overflow")
        };
        let sp2_offset = || -> u64 {
            let sp2_val = val.stack_state.sp2.unwrap().value;
            let imm = val.instruction.imm.unwrap();
            imm.checked_add(sp2_val).expect("Memory offset overflow")
        };

        let ram_write_value = || match val.memory_state {
            Some(MemoryState::Read {
                address: _,
                value: _,
            }) => panic!("Unexpected MemoryState::Read"),
            Some(MemoryState::Write {
                address: _,
                post_value,
            }) => post_value,
            None => panic!("Memory state not found"),
        };

        match val.instruction.opcode {
            WASMOpcode::I32ADD
            | WASMOpcode::I32MUL
            | WASMOpcode::I32AND
            | WASMOpcode::I32OR
            | WASMOpcode::I32XOR
            | WASMOpcode::I32EQ => [sp1_read(), sp2_read(), spd_write(), MemoryOp::noop_read()],

            // const
            WASMOpcode::I32CONST => {
                // I32Const is a special case where we only read from sp1 and write to spd
                [
                    MemoryOp::noop_read(),
                    MemoryOp::noop_read(),
                    spd_write(),
                    MemoryOp::noop_read(),
                ]
            }
            WASMOpcode::I32LOAD => [
                sp1_read(),
                MemoryOp::noop_read(),
                spd_write(),
                MemoryOp::Read(sp1_offset()),
            ],
            WASMOpcode::I32STORE => [
                sp1_read(),
                sp2_read(),
                MemoryOp::noop_write(),
                MemoryOp::Write(sp2_offset(), ram_write_value()),
            ],
            WASMOpcode::UNIMPL => {
                // This is a placeholder for unsupported opcodes
                // We return noop reads/writes
                [
                    MemoryOp::noop_read(),
                    MemoryOp::noop_read(),
                    MemoryOp::noop_write(),
                    MemoryOp::noop_read(),
                ]
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WASMInstruction {
    pub address: u64,
    pub opcode: WASMOpcode,
    pub imm: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Hash)]
#[allow(non_camel_case_types)]
pub enum WASMOpcode {
    I32CONST,
    I32AND,
    I32OR,
    I32XOR,
    I32MUL,
    I32ADD,
    I32LOAD,
    I32STORE,
    I32EQ,

    // HACK
    UNIMPL,
}

impl WASMOpcode {
    pub fn bitflag(self) -> u64 {
        1u64 << (self as u8) // TODO
    }
}

impl From<&Opcode> for WASMOpcode {
    fn from(opcode: &Opcode) -> Self {
        match opcode {
            Opcode::I32Mul => WASMOpcode::I32MUL,
            Opcode::I32Add => WASMOpcode::I32ADD,
            Opcode::I32And => WASMOpcode::I32AND,
            Opcode::I32Or => WASMOpcode::I32OR,
            Opcode::I32Xor => WASMOpcode::I32XOR,
            Opcode::I32Const(_) => WASMOpcode::I32CONST,
            Opcode::I32Load(_) => WASMOpcode::I32LOAD,
            Opcode::I32Store(_) => WASMOpcode::I32STORE,
            Opcode::I32Eq => WASMOpcode::I32EQ,

            // TODO
            Opcode::Return => WASMOpcode::UNIMPL,
            _ => panic!("Unsupported opcode for WASMOpcode conversion"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct StackState {
    pub sp: u64,
    pub sp1: Option<SPState>,
    pub sp2: Option<SPState>,
    pub spd: Option<SPState>,
}

// (address, value) tuples
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct SPState {
    pub address: u64,
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MemoryState {
    Read { address: u64, value: u64 },
    Write { address: u64, post_value: u64 },
}

/// Boolean flags used in Jolt's R1CS constraints (`opflags` in the Jolt paper).
/// Note that the flags below deviate slightly from those described in Appendix A.1
/// of the Jolt paper.
#[derive(
    Clone, Copy, Debug, PartialEq, Default, Eq, PartialOrd, Hash, Ord, EnumCountMacro, EnumIter,
)]
pub enum CircuitFlags {
    #[default] // Need a default so that we can derive EnumIter on `JoltR1CSInputs`
    StackPop,
    TwoStackPops,
    /// 1 if the instruction is a load (i.e. `LW`)
    Load,
    /// 1 if the instruction is a store (i.e. `SW`)
    Store,
    /// 1 if the lookup output is to be stored in `rd` at the end of the step.
    WriteLookupOutputToRD,
    /// Indicates whether the instruction performs a concat-type lookup.
    ConcatLookupQueryChunks,
}
pub const NUM_CIRCUIT_FLAGS: usize = CircuitFlags::COUNT;

impl WASMInstruction {
    pub fn to_circuit_flags(&self) -> [bool; NUM_CIRCUIT_FLAGS] {
        let mut flags = [false; NUM_CIRCUIT_FLAGS];

        flags[CircuitFlags::StackPop as usize] =
            !matches!(self.opcode, WASMOpcode::UNIMPL | WASMOpcode::I32CONST);

        flags[CircuitFlags::TwoStackPops as usize] = !matches!(
            self.opcode,
            WASMOpcode::UNIMPL | WASMOpcode::I32CONST | WASMOpcode::I32LOAD
        );

        flags[CircuitFlags::Load as usize] = matches!(self.opcode, WASMOpcode::I32LOAD);

        flags[CircuitFlags::Store as usize] = matches!(self.opcode, WASMOpcode::I32STORE);

        flags[CircuitFlags::WriteLookupOutputToRD as usize] = !matches!(
            self.opcode,
            WASMOpcode::UNIMPL | WASMOpcode::I32LOAD | WASMOpcode::I32STORE
        );

        flags[CircuitFlags::ConcatLookupQueryChunks as usize] = matches!(
            self.opcode,
            WASMOpcode::I32EQ | WASMOpcode::I32OR | WASMOpcode::I32XOR | WASMOpcode::I32AND
        );

        flags
    }
}
