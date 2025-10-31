pub mod utils;

pub mod ecosystems {
    pub mod ethereum {
        pub mod chain_config;
        pub mod rpc_management;
        pub mod simple_balance_query;
        pub mod provider;
        pub mod token_transfer;
        pub mod transfer;
        pub mod proxy_manager;
        pub mod proxy_commands;
    }
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
