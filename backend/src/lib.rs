#[cfg(test)]
mod tests;

pub mod admin;
pub mod api;
pub mod atlas_apps;
pub mod auth;
pub mod config;
pub mod db;
pub mod entities;
pub mod extractors;
pub mod handlers;
pub mod metrics;
pub mod middleware;
pub mod migration;
pub mod models;
pub mod services;
pub mod traits;
pub mod types;
pub mod webauthn_registry; // G-32: Axum extractors for declarative role enforcement
