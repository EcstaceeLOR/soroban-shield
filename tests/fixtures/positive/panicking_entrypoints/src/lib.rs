#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct Config;

#[contractimpl]
impl Config {
    pub fn read_config(env: Env) -> u32 {
        env.storage().instance().get(&symbol_short!("CFG")).unwrap()
    }

    pub fn must_be_set(env: Env) -> u32 {
        env.storage().instance().get(&symbol_short!("X")).expect("not set")
    }

    pub fn later(_env: Env) {
        todo!()
    }
}
