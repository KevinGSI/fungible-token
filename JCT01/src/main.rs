#![no_std]
#![no_main]

// TBD
/*
    "EARLY GAME"

Refactor code / return values for testnet & use casper-client to test currently existing functions,

Restrict mint access to admin address.
Make use of constants.rs,
move some stuff to utils.rs

    "MID GAME"

call init function to deploy a new contract,
extract further, more complex methods and structure from the CEP token standard.

    "LATE GAME"

build a client in javascript, using the SDK,
try to deploy multiple tokens and code swap logic,



*/
mod constants;
mod utils;
extern crate alloc;

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::convert::TryInto;

use casper_types::{
    account::AccountHash, contracts::NamedKeys, runtime_args, ApiError, CLType, CLValue,
    ContractHash, ContractPackageHash, ContractVersion, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Key, KeyTag, Parameter, RuntimeArgs, Tagged, URef,
};

use casper_contract::{
    contract_api::{
        runtime,
        storage::{self, dictionary_get},
    },
    unwrap_or_revert::UnwrapOrRevert,
};

use constants::*;
use utils::*;

// Jonas ERC20 token on Casper

#[no_mangle]
pub extern "C" fn Balance(
    caller_account_key: AccountHash,
    caller_account_key_as_string: &str,
) -> u64 {
    let balances_uref = get_uref("token_balances");
    let _balance = storage::dictionary_get::<u64>(balances_uref, &caller_account_key_as_string);

    let mut __balance: u64 = 0;
    match _balance {
        Ok(maybe_balance) => {
            match maybe_balance {
                Some(balance) => {
                    // Update __balance in outer scope
                    __balance = balance;
                }
                // This should never happen.
                None => {
                    // account not found, not received tokens => balance is 0.
                    __balance = 0;
                }
            }
        }
        Err(_) => {
            // This should never happen, could happen if initialization failed.
            runtime::revert(ApiError::Unhandled)
        }
    }
    __balance
    // u64
}

#[no_mangle]
pub extern "C" fn mint() {
    // Account
    let caller_account_key: AccountHash = runtime::get_caller();
    let caller_account_key_as_string = caller_account_key.to_string();

    // to be done: add permissions so that only the owner can mint.
    let circulating_supply_uref: URef = get_uref("circulating_supply");
    let circulating_supply: u64 = storage::read_or_revert(circulating_supply_uref);
    let max_total_supply_uref: URef = get_uref("max_total_supply");
    let max_total_supply: u64 = storage::read_or_revert(max_total_supply_uref);

    let mint_amount: u64 = 100;
    let balances_uref = get_uref("token_balances");

    // Is the max_supply exceeded by this mint ? - if so, revert.
    if circulating_supply + mint_amount > max_total_supply {
        runtime::revert(ApiError::PermissionDenied)
    }

    // Add mint_amount to account balance.
    let balance_before_mint: u64 = Balance(caller_account_key, &caller_account_key_as_string);
    let balance_after_mint: u64 = balance_before_mint + mint_amount;

    // First update Balance to prevent multiple execution attacks
    // TBD: make an external function to update balances / overwrite keys in dicts.
    storage::dictionary_put(
        balances_uref,
        &caller_account_key_as_string,
        balance_after_mint,
    );

    let updated_circulating_supply: u64 = circulating_supply + mint_amount;
    storage::write(circulating_supply_uref, updated_circulating_supply);
    let balance_after_mint_uref = storage::new_uref(balance_after_mint);
    // put key in runtime for testing.
    // In production this would probably be replaced by a return value.
    runtime::put_key("balance_after_mint", balance_after_mint_uref.into());
    // nothing
}

pub extern "C" fn burn() {
    // Account
    let caller_account_key: AccountHash = runtime::get_caller();
    let caller_account_key_as_string = caller_account_key.to_string();

    let circulating_supply_uref: URef = get_uref("circulating_supply");
    let circulating_supply: u64 = storage::read_or_revert(circulating_supply_uref);
    let max_total_supply_uref: URef = get_uref("max_total_supply");
    let max_total_supply: u64 = storage::read_or_revert(max_total_supply_uref);
    // To be parsed as a runtime arg later.
    let burn_amount: u64 = 25; // burn 25 tokens
    let balances_uref = get_uref("token_balances");

    let balance_before_burn: u64 = Balance(caller_account_key, &caller_account_key_as_string);
    if balance_before_burn < burn_amount {
        runtime::revert(ApiError::None)
    }
    let balance_after_burn: u64 = balance_before_burn - burn_amount;
    let updated_circulating_supply: u64 = circulating_supply - burn_amount;

    // Update Balance first to prevent multiple execution attacks.
    storage::dictionary_put(
        balances_uref,
        &caller_account_key_as_string,
        balance_after_burn,
    );

    storage::write(circulating_supply_uref, updated_circulating_supply);
    // nothing
}

#[no_mangle]
pub extern "C" fn balanceOf() -> u64 {
    let caller_account_key: AccountHash = runtime::get_caller();
    let caller_account_key_as_string = caller_account_key.to_string();

    let __balance: u64 = Balance(caller_account_key, &caller_account_key_as_string);
    // let balance_uref = storage::new_uref(__balance);
    // runtime::put_key("balance", balance_uref.into());
    __balance
    // u64
}

#[no_mangle]
pub extern "C" fn call() {
    // Constants for testing:
    let amount_mints: u64 = 10; // given a max supply of 1000 and a mint_amount of 100, 10 is the maximum possible.
                                // any value above 10 will cause a revert with a PermissionDenied error.

    // initialize token -> later to be moved to an initialization function.
    storage::new_dictionary("token_balances");
    runtime::put_key(
        "circulating_supply",
        storage::new_uref("circulating_supply").into(),
    );
    let circulating_supply_uref: URef = get_uref("circulating_supply");
    let circulating_supply: u64 = 0;
    storage::write(circulating_supply_uref, circulating_supply);
    runtime::put_key(
        "max_total_supply",
        storage::new_uref("max_total_supply").into(),
    );
    let max_total_supply_uref: URef = get_uref("max_total_supply");
    let max_total_supply: u64 = 1000;
    storage::write(max_total_supply_uref, max_total_supply);
    // now the token is initialized.

    // actions taken for testing.
    /*
    for i in (0..amount_mints) {
        mint();
        burn();
    }
    balanceOf()
    */

    // Entry Points ( for testing ) => to be moved to init function in the future
    let entry_points = {
        // Define and assign entry points for this smart contract
        let mut entry_points = EntryPoints::new();
        let mint = EntryPoint::new(
            "mint",
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        let balance = EntryPoint::new(
            "balanceOf",
            vec![],
            CLType::U64,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        let burn = EntryPoint::new(
            "burn",
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        entry_points.add_entry_point(mint);
        entry_points.add_entry_point(balance);
        entry_points.add_entry_point(burn);

        entry_points
    };

    let named_keys = {
        let mut named_keys = NamedKeys::new();
        named_keys.insert("installer".to_string(), runtime::get_caller().into());
        named_keys
    };

    storage::new_contract(
        entry_points,
        Some(named_keys),
        Some("hash_key".to_string()),
        Some("access_key".to_string()),
    );
}
