//!
//! The [`config`] module defines the configuration
//! subsystem for the proxy, including the configuration
//! schema and a lock-free hot-reload mechanism.
//! It provides the [`Config`] data model and the
//! [`ConfigManager`], which maintains the active
//! configuration using an [`ArcSwap`] container.
//!
pub mod config;
pub mod loader;
pub mod manager;
pub mod store;
pub mod watcher;
