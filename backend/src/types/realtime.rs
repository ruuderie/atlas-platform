//! Canonical types for G-07 realtime / WebSocket room messages.
//!
//! `WsMessageType` replaces raw `String` discriminators on
//! `atlas_ws_messages.message_type`. Persist via `Display`; parse with
//! `TryFrom` / `FromStr` at the boundary.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Message kind within an `atlas_ws_room`.
///
/// Stored as VARCHAR in `atlas_ws_messages.message_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WsMessageType {
    /// Ordinary user/participant text.
    Text,
    /// System event (e.g. thread closed).
    System,
    /// Operator reply visible to the Folio user.
    OperatorReply,
    /// Operator-only internal note — never returned on Folio message lists.
    InternalNote,
}

impl WsMessageType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::System => "system",
            Self::OperatorReply => "operator_reply",
            Self::InternalNote => "internal_note",
        }
    }

    /// Whether Folio end-users may see this message.
    pub fn is_user_visible(self) -> bool {
        !matches!(self, Self::InternalNote)
    }
}

impl fmt::Display for WsMessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for WsMessageType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(Self::Text),
            "system" => Ok(Self::System),
            "operator_reply" => Ok(Self::OperatorReply),
            "internal_note" => Ok(Self::InternalNote),
            other => Err(format!("unknown WsMessageType: '{other}'")),
        }
    }
}

impl TryFrom<&str> for WsMessageType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::from_str(s)
    }
}

impl TryFrom<String> for WsMessageType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}
