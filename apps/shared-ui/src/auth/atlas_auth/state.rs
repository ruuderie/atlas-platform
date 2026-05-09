#[derive(Clone, PartialEq, Debug)]
pub enum AuthStep {
    Passkey,
    Email,
    MagicLinkSent,
}

#[derive(Clone, PartialEq, Debug)]
pub enum AuthState {
    Authenticated,
    Unauthenticated,
    Authenticating,
}
