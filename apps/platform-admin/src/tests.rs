#![cfg(test)]
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn simple_test() {
    assert_eq!(1 + 1, 2);
}
