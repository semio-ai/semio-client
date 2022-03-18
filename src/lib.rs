#![feature(more_qualified_paths)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Token {
  pub token: String,
  pub expires_at: DateTime<Utc>,
}

pub mod context;
pub mod module;
pub mod common;
pub mod folder;
pub mod animation;
pub mod enumeration;
pub mod structure;
pub mod organization;
pub mod user;
pub mod store;
