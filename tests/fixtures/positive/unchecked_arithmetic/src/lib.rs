#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct Vault;

#[contractimpl]
impl Vault {
    pub fn add_funds(_env: Env, balance: i128, amount: i128) -> i128 {
        balance + amount
    }

    pub fn narrow(_env: Env, amount: u128) -> u64 {
        amount as u64
    }
}
