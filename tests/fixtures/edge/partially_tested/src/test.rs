#![cfg(test)]
use super::*;

#[test]
fn only_set_is_covered() {
    let client = CounterClient::new();
    assert_eq!(client.set_value(&admin(), &7), ());
}
