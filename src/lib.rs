#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unused_crate_dependencies)]
#![allow(unused_variables)]
#![recursion_limit = "750"]

mod compiler;
mod types;
mod vm;

extern crate alloc;
extern crate core;

pub use compiler::*;
use libm as _;
pub use types::*;
pub use vm::*;
pub use wasmparser::{FuncType, ValType};

pub struct WASMArgs {
    pub bytecode: Vec<u8>,
    pub entry_point: String,
    pub inputs: Vec<u64>,
}

pub fn trace(args: &WASMArgs) -> Vec<WASMTraceRow> {
    // TODO: Fix duplicate config code betwen decode and trace functions
    let config = CompilationConfig::default()
        .with_entrypoint_name(args.entry_point.clone().into())
        .with_allow_malformed_entrypoint_func_type(true)
        .with_consume_fuel(false);
    let (rwasm_module, _) = RwasmModule::compile(config, &args.bytecode).unwrap();
    let mut store = Store::new(ExecutorConfig::default(), ());
    let mut engine = ExecutionEngine::new();
    for input in args.inputs.iter() {
        engine.value_stack().push((*input).into());
    }
    engine.execute(&mut store, &rwasm_module).unwrap();
    let mut rows = store.jolt_tracer.rows.try_borrow_mut().unwrap();
    let mut output = Vec::new();
    output.append(&mut rows);
    drop(rows);
    output
}

pub fn decode(bytecode: &[u8], entry_point: &str) -> (Vec<WASMInstruction>, Vec<(u64, u8)>) {
    const PREPEND_NOOP: u64 = 1;
    let config = CompilationConfig::default()
        .with_entrypoint_name(entry_point.into())
        .with_allow_malformed_entrypoint_func_type(true)
        .with_consume_fuel(false);
    let (rwasm_module, _) = RwasmModule::compile(config, bytecode).unwrap();
    println!("WASM Module: {:#?}", rwasm_module);
    let instructions: Vec<WASMInstruction> = rwasm_module
        .code_section
        .instr
        .iter()
        .enumerate()
        .map(|(address, i)| i.trace(address as u64 + PREPEND_NOOP))
        .collect();
    let init_memory = rwasm_module
        .data_section
        .iter()
        .enumerate()
        .map(|(address, value)| (address as u64, *value))
        .collect();
    (instructions, init_memory)
}
