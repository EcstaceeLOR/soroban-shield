#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct Hello;

#[contractimpl]
impl Hello {
    pub fn hello(_env: Env, to: Symbol) -> Symbol {
        to
    }
}
