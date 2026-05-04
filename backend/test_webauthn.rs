use webauthn_rs::prelude::*;
fn test(w: &webauthn_rs::Webauthn) {
    let _ = w.start_discoverable_authentication();
}
