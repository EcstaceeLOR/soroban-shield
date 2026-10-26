#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, symbol_short, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    Missing = 1,
}

#[contract]
pub struct Config;

#[contractimpl]
impl Config {
    pub fn read_config(env: Env) -> Result<u32, Error> {
        env.storage().instance().get(&symbol_short!("CFG")).ok_or(Error::Missing)
    }

    pub fn read_or_default(env: Env) -> u32 {
        env.storage().instance().get(&symbol_short!("CFG")).unwrap_or(0)
    }
}
