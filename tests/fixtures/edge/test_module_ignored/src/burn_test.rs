#![cfg(test)]
use super::*;

#[test]
fn exercises_entrypoints() {
    let client = Client::new();
    client.burn();
}
