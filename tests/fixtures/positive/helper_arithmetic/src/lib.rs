#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct Vault;

fn add_fee(amount: i128) -> i128 {
    amount + 10
}

#[contractimpl]
impl Vault {
    pub fn deposit(_env: Env, amount: i128) -> i128 {
        add_fee(amount)
    }
}
