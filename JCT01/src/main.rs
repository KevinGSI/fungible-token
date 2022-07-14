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
    contracts::NamedKeys, runtime_args, CLType, CLValue, ContractHash, ContractPackageHash,
    ContractVersion, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key, KeyTag,
    Parameter, RuntimeArgs, Tagged, URef, account::AccountHash, ApiError
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
pub extern "C" fn mint(){
    let mint_amount: u64 = 100;
    let caller_account_key: AccountHash = runtime::get_caller();
    let caller_account_key_as_string = caller_account_key.to_string();
    let balances_uref = get_uref("token_balances");

    let caller_current_balance = storage::dictionary_get::<u64>(balances_uref, &caller_account_key_as_string);
    match caller_current_balance{
        Ok(maybe_balance_before_mint) => {
            match maybe_balance_before_mint{
                Some(balance_before_mint) => {
                    let updated_balance: u64 = balance_before_mint + mint_amount;
                    storage::dictionary_put(balances_uref, &caller_account_key_as_string, updated_balance);
                },
                // This should never happen.
                None => {
                    // If there is an unknown error, set balance to 0.
                    let zero: u64 = 100;
                    storage::dictionary_put(balances_uref, &caller_account_key_as_string, zero);
                }
            }
        },
        // This should happen when user has not yet received any tokens / no entry in balances.
        Err(_) => {
            // initialize account with a balance of mint_amount.
            storage::dictionary_put(balances_uref, &caller_account_key_as_string, mint_amount);
        }

    }

}

#[no_mangle]
pub extern "C" fn balanceOf(){
    let caller_account_key: AccountHash = runtime::get_caller();
    let caller_account_key_as_string = caller_account_key.to_string();
    let balances_uref = get_uref("token_balances");
    let _balance = storage::dictionary_get::<u64>(balances_uref, &caller_account_key_as_string);
    match _balance{
        Ok(maybe_balance) => {
            match maybe_balance{
                Some(balance) => {
                    let balance_uref = storage::new_uref(balance);
                    runtime::put_key("balance", balance_uref.into())
                },
                // This should never happen.
                None => {
                    // account not found, not received tokens => balance is 0.
                    let balance:u64 = 0;
                    let balance_uref = storage::new_uref(balance);
                    runtime::put_key("balance", balance_uref.into())
                }
            }
        },
        Err(_) => {
            // This should never happen, could happen if initialization failed.
        }
    }
}

#[no_mangle]
pub extern "C" fn call(){
    // initialize token
    storage::new_dictionary("token_balances");
    // currently not used for testing.

    runtime::put_key("circulating_supply", storage::new_uref("circulating_supply").into());

    let circulating_supply_uref = get_uref("circulating_supply");
    let circulating_supply: u64 = 1000;
    
    
    storage::write(circulating_supply_uref, circulating_supply);
    let circ_supply: u64 = storage::read_or_revert(circulating_supply_uref);

    let compare_to = 20;
    if circ_supply != compare_to{
        runtime::revert(ApiError::PermissionDenied)
    }
    

    // call dummy mint function
    mint();
    mint();
    mint();
    mint();
    balanceOf()
}
