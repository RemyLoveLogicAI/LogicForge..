//! Integration modules for external services

pub mod cloudflare;
pub mod genspark;
pub mod github;
pub mod vercel;

pub use cloudflare::CloudflareClient;
pub use genspark::{GenSparkClient, GenSparkStatus};
pub use github::{FileContent, GitHubClient, GitHubUser, Repository};
pub use vercel::{EnvironmentVariable, VercelClient, VercelDeployment, VercelProject};
