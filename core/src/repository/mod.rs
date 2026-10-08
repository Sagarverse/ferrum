//! Content-addressable repository engine.
//!
//! Provides content-defined chunking, deduplication, compression,
//! and a flat-file chunk store organized by hash prefix.

pub mod chunk;
pub mod dedup;
pub mod compress;
pub mod store;
pub mod manifest;
pub mod gc;

pub use chunk::*;
pub use dedup::*;
pub use compress::*;
pub use store::*;
pub use manifest::*;
pub use gc::*;
