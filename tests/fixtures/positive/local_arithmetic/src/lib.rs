#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct Vault;

#[contractimpl]
impl Vault {
    // Arithmetic on a local copy of a parameter is tracked.
    pub fn add_via_local(_env: Env, balance: i128) -> i128 {
        let total = balance;
        total + 1
    }
}
