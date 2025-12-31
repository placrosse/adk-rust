//! SSO and OAuth/OIDC integration for adk-auth.
//!
//! This module provides JWT validation, OIDC provider support, and SSO integration.
//!
//! # Features
//!
//! Enable the `sso` feature to use these modules:
//!
//! ```toml
//! [dependencies]
//! adk-auth = { version = "0.1", features = ["sso"] }
//! ```
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use adk_auth::sso::{JwtValidator, TokenClaims};
//!
//! let validator = JwtValidator::builder()
//!     .issuer("https://accounts.google.com")
//!     .audience("your-client-id")
//!     .build()?;
//!
//! let claims = validator.validate(token).await?;
//! println!("User: {}", claims.sub);
//! ```

mod claims;
mod error;
mod jwks;
mod validator;

pub use claims::TokenClaims;
pub use error::TokenError;
pub use jwks::JwksCache;
pub use validator::{JwtValidator, JwtValidatorBuilder, TokenValidator};

// Re-export providers when available
#[cfg(feature = "sso")]
mod providers;
#[cfg(feature = "sso")]
pub use providers::*;
