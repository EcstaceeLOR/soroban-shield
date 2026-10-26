#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

fn only_admin(admin: &Address) {
    admin.require_auth();
}

#[contract]
pub struct Counter;

#[contractimpl]
impl Counter {
    pub fn set_value(env: Env, admin: Address, value: u32) {
        only_admin(&admin);
        env.storage().instance().set(&symbol_short!("VALUE"), &value);
    }
}

#[cfg(test)]
mod test;
