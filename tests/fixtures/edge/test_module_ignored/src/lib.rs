#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn burn(env: Env, from: Address) {
        from.require_auth();
        env.storage().persistent().remove(&from);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[contract]
    pub struct Harness;

    #[contractimpl]
    impl Harness {
        pub fn unguarded(env: Env, from: Address) {
            env.storage().persistent().set(&from, &1u32);
        }
    }
}

#[cfg(test)]
mod burn_test;
