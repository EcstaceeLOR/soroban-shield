#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct Vault;

#[contractimpl]
impl Vault {
    // Known false negative: a value read from storage is not tracked, only
    // values derived from integer parameters are.
    pub fn bump(env: Env) -> i128 {
        let balance: i128 = env.storage().instance().get(&symbol_short!("BAL")).unwrap_or(0);
        balance + 1
    }
}
