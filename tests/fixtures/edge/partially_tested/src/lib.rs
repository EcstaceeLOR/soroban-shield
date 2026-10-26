#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct Counter;

#[contractimpl]
impl Counter {
    pub fn set_value(env: Env, admin: Address, value: u32) {
        admin.require_auth();
        env.storage().instance().set(&symbol_short!("VALUE"), &value);
    }

    pub fn clear(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().remove(&symbol_short!("VALUE"));
    }
}

#[cfg(test)]
mod test;
