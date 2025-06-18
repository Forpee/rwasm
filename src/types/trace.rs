use crate::Opcode;
use serde::{Deserialize, Serialize};

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
