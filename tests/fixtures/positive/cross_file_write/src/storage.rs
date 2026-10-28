use soroban_sdk::{symbol_short, Env, Symbol};

const VALUE: Symbol = symbol_short!("VALUE");

pub fn write_value(env: &Env, value: u32) {
    env.storage().instance().set(&VALUE, &value);
}

pub fn read_value(env: &Env) -> u32 {
    env.storage().instance().get(&VALUE).unwrap_or(0)
}
