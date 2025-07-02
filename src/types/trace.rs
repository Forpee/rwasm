use crate::Opcode;
use serde::{Deserialize, Serialize};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WASMTraceRow {
    pub instruction: WASMInstruction,
    pub stack_state: StackState,
    pub memory_state: Option<MemoryState>,
    pub advice_value: Option<u64>,
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
    I32MUL,
    I32ADD,
    I32SUB,
}

impl WASMOpcode {
    pub fn bitflag(self) -> u64 {
        1u64 << (self as u8)
    }
}

impl From<&Opcode> for WASMOpcode {
    fn from(opcode: &Opcode) -> Self {
        match opcode {
            Opcode::I32Mul => WASMOpcode::I32MUL,
            Opcode::I32Add => WASMOpcode::I32ADD,
            Opcode::I32Sub => WASMOpcode::I32SUB,
            _ => panic!("Unsupported opcode for WASMOpcode conversion"),
        }
    }
}

// (address, value) tuples
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StackState {
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
