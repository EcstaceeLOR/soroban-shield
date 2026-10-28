#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&admin, &1u32);
    }

    pub fn burn(env: Env, from: Address) {
        from.require_auth();
        env.storage().persistent().remove(&from);
    }
}

#[cfg(test)]
mod test;
