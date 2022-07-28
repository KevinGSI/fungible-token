#![no_std]
#![no_main]
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
fn Balance(caller_account_hash: AccountHash, caller_account_hash_as_string: &str) -> u64 {
    let balances_key: Key = match runtime::get_key("holdings") {
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey),
    };

    let _balance = storage::dictionary_get::<u64>(
        balances_key.into_uref().unwrap_or_revert(),
        &caller_account_hash_as_string,
    );

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
// account owner should be changed to a contract_hash
pub extern "C" fn updateOwner() {
    let new_owner: AccountHash = runtime::get_named_arg("new_owner");
    let owner_account_uref: URef = match runtime::get_key("owner") {
        Some(uref) => uref,
        None => runtime::revert(ApiError::MissingKey),
    }
    .into_uref()
    .unwrap_or_revert();
    let owner_account: AccountHash = storage::read_or_revert(owner_account_uref);
    if runtime::get_caller() != owner_account {
        runtime::revert(ApiError::PermissionDenied);
    }
    storage::write(owner_account_uref, new_owner);
}

// Fully functional.
#[no_mangle]
pub extern "C" fn mint() {
    // Account
    let caller_account_hash: AccountHash = runtime::get_caller();
    let caller_account_hash_as_string = caller_account_hash.to_string();
    // TO BE ADDED: OWNER VALIDATION CHECK

    // Read current owner of the contract and revert if the caller isn't eligible to mint()
    let owner_account_uref: URef = match runtime::get_key("owner") {
        Some(uref) => uref,
        None => runtime::revert(ApiError::MissingKey),
    }
    .into_uref()
    .unwrap_or_revert();
    let owner_account: AccountHash = storage::read_or_revert(owner_account_uref);
    if runtime::get_caller() != owner_account {
        runtime::revert(ApiError::PermissionDenied); // only owner account is eligible
    }
    // to be done: add permissions so that only the owner can mint.

    // CIRCULATING SUPPLY
    let circulating_supply_uref: URef = match runtime::get_key("circ") {
        Some(uref) => uref,
        None => runtime::revert(ApiError::MissingKey),
    }
    .into_uref()
    .unwrap_or_revert();
    let circulating_supply: u64 = storage::read_or_revert(circulating_supply_uref);

    // TOTAL SUPPLY
    let max_total_supply_uref: URef = match runtime::get_key("maxsupp") {
        Some(uref) => uref,
        None => runtime::revert(ApiError::MissingKey),
    }
    .into_uref()
    .unwrap_or_revert();
    let max_total_supply: u64 = storage::read_or_revert(max_total_supply_uref);

    // MINT AMOUNT
    let mint_amount: u64 = 100;

    // Is the max_supply exceeded by this mint ? - if so, revert.
    if circulating_supply + mint_amount > max_total_supply {
        runtime::revert(ApiError::PermissionDenied)
    }

    // BALANCE

    let balances_key: Key = match runtime::get_key("holdings") {
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey),
    };
    let balances_uref: URef = balances_key.into_uref().unwrap_or_revert();
    /*let _balance = storage::dictionary_get::<u64>(
        balances_key.into_uref().unwrap_or_revert(),
        &caller_account_hash_as_string,
    );*/
    // Add mint_amount to account balance.
    let balance_before_mint: u64 = Balance(caller_account_hash, &caller_account_hash_as_string);
    let balance_after_mint: u64 = balance_before_mint + mint_amount;

    // First update Balance to prevent multiple execution attacks
    // TBD: make an external function to update balances / overwrite keys in dicts.
    storage::dictionary_put(
        balances_uref,
        &caller_account_hash_as_string,
        balance_after_mint,
    );

    let updated_circulating_supply: u64 = circulating_supply + mint_amount;
    storage::write(circulating_supply_uref, updated_circulating_supply);
}

#[no_mangle]
pub extern "C" fn burn() {
    let burn_amount: u64 = 10;

    // TBD: move Uref conversion into a function if possible.
    let balances_key: Key = match runtime::get_key("holdings") {
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey),
    };
    // balances dictionary UREF
    let balances_uref: URef = balances_key.into_uref().unwrap_or_revert();
    let circulating_supply_uref: URef = match runtime::get_key("circ") {
        Some(uref) => uref,
        None => runtime::revert(ApiError::MissingKey),
    }
    .into_uref()
    .unwrap_or_revert();
    // circulating supply
    let circulating_supply: u64 = storage::read_or_revert(circulating_supply_uref);

    let caller_account_hash = runtime::get_caller();
    let caller_account_hash_as_string = caller_account_hash.to_string();
    // Account Balance before burn.
    let balance_before_burn: u64 = Balance(caller_account_hash, &caller_account_hash_as_string);
    if balance_before_burn - burn_amount < 0 {
        runtime::revert(ApiError::PermissionDenied); // incorrect ApiError, create one for balance too low.
    }
    let balance_after_burn: u64 = balance_before_burn - burn_amount;
    storage::dictionary_put(
        balances_uref,
        &caller_account_hash_as_string,
        balance_after_burn,
    );

    let updated_circulating_supply: u64 = circulating_supply - burn_amount;
    storage::write(circulating_supply_uref, updated_circulating_supply);
}

#[no_mangle]
// this can only be called by another contract and does not return a value to the cli.
pub extern "C" fn balanceOf() -> u64 {
    let caller_account_hash: AccountHash = runtime::get_caller();
    let caller_account_hash_as_string = caller_account_hash.to_string();
    let _balance: u64 = Balance(caller_account_hash, &caller_account_hash_as_string);
    runtime::ret(CLValue::from_t(_balance).unwrap_or_revert());
    // u64
}

#[no_mangle]
pub extern "C" fn call() {
    // initialize token -> later to be moved to an initialization function.

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

        let updateOwner = EntryPoint::new(
            "updateOwner",
            // AccountHash should be type any and then parsed as account hash
            vec![Parameter::new("updated_owner_account", CLType::Any)], // not sure if this type is correct.
            CLType::Unit,
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
        entry_points.add_entry_point(updateOwner);
        entry_points.add_entry_point(burn);
        entry_points
    };

    let named_keys = {
        let mut named_keys = NamedKeys::new();
        named_keys.insert("installer".to_string(), runtime::get_caller().into());
        // Warning: if key exists on different contract, deploy will fail ? to be investigated.
        let balances_dict = storage::new_dictionary("holdings").unwrap_or_revert();
        named_keys.insert("holdings".to_string(), balances_dict.into());

        let owner_account: URef = storage::new_uref("owner");
        named_keys.insert("owner".to_string(), owner_account.into());
        let owner_value: AccountHash = runtime::get_caller();
        storage::write(owner_account, owner_value);

        let circulating_supply = storage::new_uref("circ");
        named_keys.insert("circ".to_string(), circulating_supply.into());
        let circulating_supply_value: u64 = 0;
        storage::write(circulating_supply, circulating_supply_value);

        let max_total_supply = storage::new_uref("maxsupp");
        named_keys.insert("maxsupp".to_string(), max_total_supply.into());
        let max_total_supply_value: u64 = 1000000;
        storage::write(max_total_supply, max_total_supply_value);

        named_keys
    };

    storage::new_contract(
        entry_points,
        Some(named_keys),
        Some("JCT_val_v.0.0.1".to_string()),
        Some("access_key".to_string()),
    );
}
