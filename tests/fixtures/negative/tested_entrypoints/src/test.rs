#![cfg(test)]
use super::*;

#[test]
fn set_then_clear() {
    let client = CounterClient::new();
    assert_eq!(client.set_value(&admin(), &7), ());
    client.clear(&admin());
}
