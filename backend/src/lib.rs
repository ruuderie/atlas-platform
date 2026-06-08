#[cfg(test)]
mod tests;

pub mod types;
pub mod api;
pub mod auth;
pub mod webauthn_registry;
pub mod db;
pub mod entities;
pub mod migration;
pub mod middleware;
pub mod handlers;
pub mod models;
pub mod admin;
pub mod traits;
pub mod config;
pub mod services;
pub mod atlas_apps;
pub mod metrics;
pub mod extractors; // G-32: Axum extractors for declarative role enforcement
