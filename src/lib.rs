
#![allow(dead_code)]

mod utils;
mod mavm;
mod pos;
mod uint256;
mod stringtable;

use wasm_bindgen::prelude::*;
use crate::utils::{process_wasm, has_label, get_inst, resolve_labels};
use crate::mavm::{Label,Value};
use crate::uint256::{Uint256};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn read_buffer(idx: i32) -> i32;
    fn setlen(idx: i32);
    fn getlen() -> i32;
    fn write_buffer(idx: i32, c: i32);
    fn usegas(gas: i32);
}

#[wasm_bindgen]
pub fn test() -> u32 {
    let mut input = vec![];
    let input_len = getlen();
    for i in 0..input_len {
        input.push(read_buffer(i) as u8)
    }
    usegas(input_len / 10 + 1);

    let ops = process_wasm(&input);
    let (res_ops, _) = resolve_labels(ops.clone());

    let mut output = vec![];

    for (idx, op) in res_ops.iter().enumerate() {
        let inst = get_inst(&op);
        output.push(inst);
        match &op.immediate {
            None => output.push(0),
            Some (Value::Int(a)) => {
                output.push(1);
                let bytes = a.to_bytes_be();
                for i in 0..8 {
                    output.push(bytes[i+24])
                }
            },
            Some (Value::Tuple(tup)) => {
                if tup.len() == 5 {
                    output.push(2) // main env
                } else if tup.len() == 2 {
                    output.push(3) // main env
                } else {
                    panic!("bad immed")
                }
            },
            _ => {
                panic!("bad immed")
            }
        }
        if has_label(&ops[idx]) {
            output.push(1)
        } else {
            output.push(0)
        }
        /*
        match ops[idx].clone().get_label() {
            None => output.push(0),
            Some(Label::Evm(num)) => {
                output.push(1);
                let bytes = Uint256::from_usize(*num).to_bytes_be();
                for i in 0..8 {
                    output.push(bytes[i+24])
                }
            }
            Some(Label::WasmFunc(num)) => {
                output.push(2)
            }
            _ => {
                panic!("bad label")
            }
        }*/
    };

    for i in 0..output.len() {
        write_buffer(i as i32, output[i as usize] as i32)
    };
    setlen(output.len() as i32);

    0

}

/*
#[wasm_bindgen]
pub fn test() -> u32 {

    let input = hex::decode("0061736d0100000001060160017f017f03020100070801046d61696e00000a070105002000690b").unwrap();

    let v = load(&input);

    /*
    for i in 0..32 {
        write_buffer(i, output[i as usize] as i32)
    };
    */
    setlen(v.len() as i32);

    0

}
*/
