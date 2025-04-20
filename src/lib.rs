// Export our core modules
pub mod core;
pub mod platform;

// Legacy exports to maintain backward compatibility during transition
pub mod block {
    pub use crate::core::block::*;
}

pub mod player {
    pub use crate::core::player::*;
}

pub mod game {
    pub use crate::core::game::*;
}
