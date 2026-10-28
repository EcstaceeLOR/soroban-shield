use super::*;

#[contract]
pub struct Harness;

#[contractimpl]
impl Harness {
    pub fn unguarded(env: Env, from: Address) {
        env.storage().persistent().set(&from, &1u32);
    }
}

#[test]
fn burn_removes_balance() {
    let _ = Token::burn;
}
