use wasm_bindgen::prelude::*;
use serde_json::Value;

#[wasm_bindgen(module = "/src/auth/passkey.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub async fn createPasskeyBinding(options_json: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn getPasskeyBinding(options_json: &str) -> Result<JsValue, JsValue>;
}

pub async fn start_registration(options: &Value) -> Result<Value, String> {
    let options_json = serde_json::to_string(options).map_err(|e| e.to_string())?;
    
    let result = createPasskeyBinding(&options_json)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Browser declined passkey creation".to_string()))?;
        
    let result_str = result.as_string().unwrap_or_default();
    serde_json::from_str(&result_str).map_err(|e| e.to_string())
}

pub async fn start_authentication(options: &Value) -> Result<Value, String> {
    let options_json = serde_json::to_string(options).map_err(|e| e.to_string())?;
    
    let result = getPasskeyBinding(&options_json)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Browser cancelled passkey authentication".to_string()))?;
        
    let result_str = result.as_string().unwrap_or_default();
    serde_json::from_str(&result_str).map_err(|e| e.to_string())
}
