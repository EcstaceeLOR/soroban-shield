#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct Vault;

#[contractimpl]
impl Vault {
    pub fn add_funds(_env: Env, balance: i128, amount: i128) -> Option<i128> {
        balance.checked_add(amount)
    }

    pub fn narrow(_env: Env, amount: u128) -> Option<u64> {
        u64::try_from(amount).ok()
    }

    pub fn widen(_env: Env, small: u32) -> u64 {
        small as u64
    }
}
