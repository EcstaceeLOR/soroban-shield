#![no_std]
mod storage;

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct Counter;

#[contractimpl]
impl Counter {
    pub fn set_value(env: Env, value: u32) {
        storage::write_value(&env, value);
    }

    pub fn get_value(env: Env) -> u32 {
        storage::read_value(&env)
    }
}
