#![no_std]
use soroban_sdk::{contract, contractimpl, panic_with_error, symbol_short, Address, Env, Symbol, Val, Vec};

#[contract]
pub struct Router;

#[contractimpl]
impl Router {
    pub fn forward(env: Env, func: Symbol, args: Vec<Val>) -> Val {
        let target: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("TARGET"))
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotConfigured));
        env.invoke_contract(&target, &func, args)
    }

    pub fn ping(env: Env) -> bool {
        let peer = env.current_contract_address();
        let client = PeerClient::new(&env, &peer);
        client.try_ping().is_ok()
    }
}
