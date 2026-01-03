// trailbase/mod.rs - TrailBase backend integration module
//
// This module provides a trait-based architecture for syncing entities
// with TrailBase backend. Supports offline-first sync with conflict resolution.

pub mod client;
pub mod models;
pub mod sync;

pub use client::TrailBaseClient;
