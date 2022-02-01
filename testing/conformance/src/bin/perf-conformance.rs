// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::env::var;
use std::ffi::CString;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use conformance_tests::vector::{MessageVector, Selector, TestVector, Variant};
use conformance_tests::vm::{TestKernel, TestMachine};
use fvm::executor::{ApplyKind, DefaultExecutor, Executor};
use fvm_shared::address::Protocol;
use fvm_shared::blockstore::MemoryBlockstore;
use fvm_shared::crypto::signature::SECP_SIG_LEN;
use fvm_shared::encoding::Cbor;
use fvm_shared::message::Message;
use ittapi_rs::*;

fn main() {
    let my_path = match var("VECTOR") {
        Ok(v) => Path::new(v.as_str()).to_path_buf(),
        Err(_) => panic!("what are you perfing??"),
    };

    let file = File::open(&my_path).unwrap();
    let reader = BufReader::new(file);
    let vector: TestVector = serde_json::from_reader(reader).unwrap();

    let domain_cstring = CString::new("profile_conformance").unwrap();
    let handle_cstring = CString::new(format!("{:?}", my_path)).unwrap();

    let itt_info = unsafe {
        (
            __itt_domain_create(domain_cstring.as_ptr()),
            __itt_string_handle_create(handle_cstring.as_ptr()),
        )
    };
    let TestVector::Message(vector) = vector;
    let skip = !vector.selector.as_ref().map_or(true, Selector::supported);
    if skip {
        println!("skipping because selector not supported");
        return;
    }
    let (bs, _) = async_std::task::block_on(vector.seed_blockstore()).unwrap();
    for variant in vector.preconditions.variants.iter() {
        run_variant_for_perf(bs.clone(), &vector, variant, true, itt_info)
    }
}

pub fn run_variant_for_perf(
    bs: MemoryBlockstore,
    v: &MessageVector,
    variant: &Variant,
    with_profiler: bool,
    itt_info: (*mut __itt_domain, *mut __itt_string_handle),
) {
    // Construct the Machine.
    let machine = TestMachine::new_for_vector(v, variant, bs, with_profiler);
    let mut exec: DefaultExecutor<TestKernel> = DefaultExecutor::new(machine);

    let (itt_domain, itt_handle) = itt_info;

    // Apply all messages in the vector.
    for m in v.apply_messages.iter() {
        let msg = Message::unmarshal_cbor(&m.bytes).unwrap();

        // Execute the message.
        let mut raw_length = m.bytes.len();
        if msg.from.protocol() == Protocol::Secp256k1 {
            // 65 bytes signature + 1 byte type + 3 bytes for field info.
            raw_length += SECP_SIG_LEN + 4;
        }

        unsafe {
            __itt_task_begin(itt_domain, __itt_null, __itt_null, itt_handle);
            exec.execute_message(msg, ApplyKind::Explicit, raw_length)
                .expect("failed to execute a message");
            __itt_task_end(itt_domain);
        }
    }
}