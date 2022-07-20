use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_execution_engine::core::engine_state::{
    run_genesis_request::RunGenesisRequest, GenesisAccount,
};
use casper_types::{
    account::AccountHash, runtime_args, Key, Motes, PublicKey, RuntimeArgs, SecretKey, U512,
};
use std::path::PathBuf;

// This constant defines which wasm file to load and pass to the instance of the EE
const CONTRACT_WASM: &str = "JCT01.wasm";

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
#[test]
fn _test() {
    // Initialize an instance of the execution engine and assign it to the builder variable
    let mut builder = InMemoryWasmTestBuilder::default();

    // Execute the genesis process
    builder.run_genesis(&*DEFAULT_RUN_GENESIS_REQUEST).commit();

    // Retrieve the contract wasm from the specified location and assign to the session code variable
    let session_code = PathBuf::from(CONTRACT_WASM);

    // Retrieve runtime arguments. These should be same as defined in the contract
    // This allows use to check and assert behavior of the session code
    let runtime_args = runtime_args! {};

    // Create a deploy item, which emulates the deploy being sent to the network
    // Use the host side functionality of standard payment and passes in the required runtime argument "amount" with some default value
    // Load the session wasm and pass in the runtime arguments
    // Sets up the session code to be executed in the default account using auth keys and default account address
    let deploy_item = DeployItemBuilder::new()
        .with_empty_payment_bytes(runtime_args! {
        ARG_AMOUNT => *DEFAULT_PAYMENT})
        .with_session_code(CONTRACT_WASM, runtime_args.clone())
        .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
        .with_address(*DEFAULT_ACCOUNT_ADDR)
        .build();

    // Create the execution request that will eventually be executed by the EE.
    let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

    // Invoke the EE to execute the session code that we are testing
    builder.exec(execute_request).expect_success().commit();

    // Verify the results of the execution match our expectations from the contract using the test results
    let result_key = *builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("the default account must be present")
        .named_keys()
        .get("balance")
        .expect("must have key as part of session code execution");

    let value: u64 = builder
        .query(None, result_key, &vec![])
        .expect("must have the stored value")
        .as_cl_value()
        .expect("must have some CLValue")
        .to_owned()
        .into_t()
        .expect("must convert the CLValue into a u64");
    //assert_eq!(1000, value);
    println!("{}", "------------------------");

    println!("Balance Key: {:?}", result_key);
    println!("Value at Key: {:?}", value);

    println!("{}", "------------------------");
}
