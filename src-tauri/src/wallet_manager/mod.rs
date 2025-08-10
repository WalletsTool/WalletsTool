pub mod chain_config;
pub mod utils;
pub mod rpc_management;

pub mod ecosystems {
    pub mod ethereum;
    pub mod solana;
}

// Backward compatible re-exports (existing command paths)
pub use ecosystems::ethereum::{
    provider,
    token_transfer,
    transfer,
};

// Keep legacy ethereum code available under a feature-gated module path if needed later
// pub use ecosystems::ethereum::legacy;
