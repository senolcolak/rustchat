//! Authentication module for rustchat
//!
//! Provides JWT tokens, password hashing, and auth middleware.

pub mod jwt;
pub mod middleware;
pub mod password;
pub mod policy;

pub use jwt::*;
pub use middleware::*;
pub use password::*;
pub use policy::*;
