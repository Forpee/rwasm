use crate::{Opcode, MEMORY_OPS_PER_INSTRUCTION};
use serde::{Deserialize, Serialize};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

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
        let sp1_read = || MemoryOp::Read(val.stack_state.sp1.unwrap().0);
        let sp2_read = || MemoryOp::Read(val.stack_state.sp2.unwrap().0);
        let spd_write = || {
            MemoryOp::Write(
                val.stack_state.spd.unwrap().0,
                val.stack_state.spd.unwrap().1,
            )
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
            _ => unreachable!("{val:?}"),
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
            Opcode::ReturnCallInternal(_)
            | Opcode::StackCheck(_)
            | Opcode::SignatureCheck(_)
            | Opcode::Return
            | Opcode::ConsumeFuel(_) => WASMOpcode::UNIMPL,
            _ => panic!("Unsupported opcode for WASMOpcode conversion"),
        }
    }
}

// (address, value) tuples
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StackState {
    pub sp: u64,
    pub sp1: Option<(u64, u64)>,
    pub sp2: Option<(u64, u64)>,
    pub spd: Option<(u64, u64)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MemoryState {
    Read {
        address: u64,
        value: u64,
    },
    Write {
        address: u64,
        pre_value: u64,
        post_value: u64,
    },
}

/// Boolean flags used in Jolt's R1CS constraints (`opflags` in the Jolt paper).
/// Note that the flags below deviate slightly from those described in Appendix A.1
/// of the Jolt paper.
#[derive(
    Clone, Copy, Debug, PartialEq, Default, Eq, PartialOrd, Hash, Ord, EnumCountMacro, EnumIter,
)]
pub enum CircuitFlags {
    #[default] // Need a default so that we can derive EnumIter on `JoltR1CSInputs`
    BinOp,
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

        flags[CircuitFlags::BinOp as usize] =
            !matches!(self.opcode, WASMOpcode::UNIMPL | WASMOpcode::I32CONST);

        flags[CircuitFlags::WriteLookupOutputToRD as usize] =
            !matches!(self.opcode, WASMOpcode::UNIMPL);

        flags[CircuitFlags::ConcatLookupQueryChunks as usize] = matches!(
            self.opcode,
            WASMOpcode::I32EQ | WASMOpcode::I32OR | WASMOpcode::I32XOR | WASMOpcode::I32AND
        );

        flags
    }
}
