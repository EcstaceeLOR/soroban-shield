#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val, Vec};

#[contract]
pub struct Router;

#[contractimpl]
impl Router {
    pub fn forward(env: Env, target: Address, func: Symbol, args: Vec<Val>) -> Val {
        env.invoke_contract(&target, &func, args)
    }

    pub fn ping_ignoring_failure(env: Env, peer: Address) {
        let client = PeerClient::new(&env, &peer);
        let _ = client.try_ping();
    }
}
