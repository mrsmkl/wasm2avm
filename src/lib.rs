
#![allow(dead_code)]

mod utils;
mod mavm;
mod pos;
mod uint256;
mod stringtable;

use wasm_bindgen::prelude::*;
use crate::utils::{process_wasm, has_label, get_inst, resolve_labels, simple_op};
use crate::mavm::{Label,Value,Instruction,AVMOpcode};
use crate::uint256::{Uint256};
use ethers_core::utils::keccak256;

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
    fn wextra(idx: i32, c: i32);
}

/*
fn push_int(output: &mut Vec<u8>, a: &Uint256) {
    let bytes = a.to_bytes_be();
    for i in 0..8 {
        output.push(bytes[i+24])
    }
}
*/

fn push_bytes32(output: &mut Vec<u8>, a: &Uint256) {
    let bytes = a.to_bytes_be();
    for i in 0..32 {
        output.push(bytes[i])
    }
}

fn hash_instruction(inst: &Instruction, prev_hash: &Uint256) -> Uint256 {
    match &inst.immediate {
        None => {
            let mut buf = vec![];
            buf.push(1u8);
            // println!("{:?}", inst.opcode);
            buf.push(get_inst(inst));
            push_bytes32(&mut buf, prev_hash);
            Uint256::from_bytes(&keccak256(&buf))
        }
        Some(immed) => {
            let mut buf = vec![];
            buf.push(1u8);
            buf.push(get_inst(inst));
            if let Value::Int(i) = immed.avm_hash() {
                    // println!("immed hash {}", i);
                push_bytes32(&mut buf, &i);
            }
            push_bytes32(&mut buf, prev_hash);
            // println!("hash len {}", buf.len());
            Uint256::from_bytes(&keccak256(&buf))
        }
    }
}

fn compute_hash(ops : &Vec<Instruction>) -> (Uint256, Uint256) {
    // start from errCodePoint
    let mut hash = hash_instruction(&simple_op(AVMOpcode::Zero), &Uint256::from_u64(0));
    let mut labels = vec![];
    for inst in ops.iter().rev() {
        hash = hash_instruction(inst, &hash);
        // println!("After {} hash is {}", inst, hash);
        if crate::utils::has_label(&inst) {
            // println!("Found label at {}", hash);
            labels.push(Value::HashOnly(hash.clone(), 1))
        }
    }

    // println!("Labels are here {:?}", labels);
    let mut labels_rev = vec![];
    for a in labels.iter().rev() {
        labels_rev.push(a.clone())
    }
    let tab = crate::utils::make_table(&labels_rev);
    let table_hash = if let Value::Int(i) = tab.avm_hash() {
        i
    } else {
        Uint256::from_u64(0)
    };
    // let table_hash = Uint256::from_u64(0);
    (hash, table_hash)
}

pub fn process(input: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut output = vec![];
    let mut extra = vec![];

    let ops = process_wasm(&input);
    let (res_ops, _) = resolve_labels(ops.clone());
    let ops : Vec<&Instruction> = ops.iter().rev().collect();

    let (hash, thash) = compute_hash(&res_ops);
    push_bytes32(&mut output, &hash);
    push_bytes32(&mut output, &thash);

    for (idx, op) in res_ops.iter().rev().enumerate() {
        let inst = get_inst(&op);
        extra.push(inst);
        match &op.immediate {
            None => extra.push(0),
            Some (Value::Int(a)) => {
                extra.push(1);
                push_bytes32(&mut extra, a);
            },
            Some (Value::Tuple(tup)) => {
                if tup.len() == 5 {
                    extra.push(2) // main env
                } else if tup.len() == 2 {
                    match &tup[1] {
                        Value::Int(a) => {
                            extra.push(3);
                            push_bytes32(&mut extra, &a);
                        },
                        _ => panic!("bad immed")
                    }
                } else {
                    panic!("bad immed")
                }
            },
            _ => {
                panic!("bad immed")
            }
        }
        if has_label(&ops[idx]) {
            extra.push(1)
        } else {
            extra.push(0)
        }
    };

    // println!("Op hash {}, Table hash {}, length {}", hash, thash, extra.len());

    extra.push(255);
    (output, extra)
}

#[wasm_bindgen]
pub fn test() -> u32 {
    let mut input = vec![];
    let input_len = getlen();
    for i in 0..input_len {
        input.push(read_buffer(i) as u8)
    }
    usegas(input_len / 10 + 1);

    let (output, extra) = process(&input);
    /*
    let output = input.clone();
    let extra = input.clone();
    */

    for i in 0..output.len() {
        write_buffer(i as i32, output[i as usize] as i32)
    };
    setlen(output.len() as i32);

    for i in 0..extra.len() {
        wextra(i as i32, extra[i as usize] as i32)
    };

    0

}

/*
#[wasm_bindgen]
pub fn test() -> u32 {

    let input = hex::decode("0061736d0100000001130460017f017f6000017f60027f7f0060017f00024e0403656e760d6765746c656e5f627566666572000103656e760d7365746c656e5f627566666572000303656e760b726561645f627566666572000003656e760c77726974655f627566666572000203020101070801047465737400040a0d010b00410041de01100341000b").unwrap();

    let v = process(&input);

    /*
    for i in 0..32 {
        write_buffer(i, output[i as usize] as i32)
    };
    setlen(v.len() as i32);
    */

    0

}
*/
