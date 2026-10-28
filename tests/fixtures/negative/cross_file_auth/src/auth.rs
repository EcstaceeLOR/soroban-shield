use soroban_sdk::{Address, Env};

pub fn require_admin(_env: &Env, admin: &Address) {
    admin.require_auth();
}
