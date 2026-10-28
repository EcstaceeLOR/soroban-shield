#![no_std]
mod auth;
mod storage;

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Counter;

#[contractimpl]
impl Counter {
    pub fn set_value(env: Env, admin: Address, value: u32) {
        auth::require_admin(&env, &admin);
        storage::write_value(&env, value);
    }
}
