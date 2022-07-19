use fvm::executor::{ApplyKind, Executor};
use fvm_integration_tests::tester::{Account, Tester};
use fvm_ipld_blockstore::MemoryBlockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::error::ExitCode;
use fvm_shared::message::Message;
use fvm_shared::state::StateTreeVersion;
use fvm_shared::version::NetworkVersion;
use num_traits::Zero;

const WASM_COMPILED_PATH: &str =
    "../../target/debug/wbuild/fil_nitro_adjudicator_actor/fil_nitro_adjudicator_actor.compact.wasm";

#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug, Default)]
pub struct State {
    pub value: i64,
}

// Utility function to instantiation integration tester
fn instantiate_tester() -> (Account, Tester<MemoryBlockstore>, Address) {
    // Instantiate tester
    let mut tester = Tester::new(
        NetworkVersion::V15,
        StateTreeVersion::V4,
        MemoryBlockstore::default(),
    )
    .unwrap();

    let sender: [Account; 1] = tester.create_accounts().unwrap();

    // Set actor state
    let actor_state = State::default();
    let state_cid = tester.set_state(&actor_state).unwrap();

    // Set actor
    let actor_address = Address::new_id(10000);

    // Get wasm bin
    let wasm_path = std::env::current_dir()
        .unwrap()
        .join(WASM_COMPILED_PATH)
        .canonicalize()
        .unwrap();

    let wasm_bin = std::fs::read(wasm_path).expect("Unable to read file");

    tester
        .set_actor_from_bin(&wasm_bin, state_cid, actor_address, BigInt::zero())
        .unwrap();

    (sender[0], tester, actor_address)
}

#[test]
fn say_hello() {
    // Instantiate tester
    let (sender, mut tester, actor_address) = instantiate_tester();

    // Instantiate machine
    tester.instantiate_machine().unwrap();

    // Params setup
    let x: i64 = 10000000000;
    let params = RawBytes::serialize(&x).unwrap();

    // Send message to set
    let message = Message {
        from: sender.1,
        to: actor_address,
        gas_limit: 1000000000,
        method_num: 1,
        params,
        ..Message::default()
    };

    // Set inner state value
    let res = tester
        .executor
        .as_mut()
        .unwrap()
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(ExitCode::OK, res.msg_receipt.exit_code);
}
