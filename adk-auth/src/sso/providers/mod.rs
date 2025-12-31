//! SSO provider implementations.

mod oidc;

pub use oidc::OidcProvider;

// Provider-specific implementations
mod azure;
mod google;
mod okta;

pub use azure::AzureADProvider;
pub use google::GoogleProvider;
pub use okta::OktaProvider;
