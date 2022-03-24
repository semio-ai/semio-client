#![feature(more_qualified_paths)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Token {
  pub token: String,
  pub expires_at: DateTime<Utc>,
}

pub mod animation;
pub mod common;
pub mod context;
pub mod enumeration;
pub mod folder;
pub mod module;
pub mod organization;
pub mod store;
pub mod structure;
pub mod user;
