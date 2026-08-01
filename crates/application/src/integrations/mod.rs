//! Integration adapters: script AI + image generation.
//! Default = local stubs/heuristics. Real APIs: set keys in Settings and enable provider.
//! Wiring a new API = implement the trait + register in the catalog (no UI rewrite).

pub mod budget;
pub mod config;
pub mod image_gen;
pub mod omniroute;
pub mod script_ai;

pub use budget::*;
pub use config::*;
pub use image_gen::*;
pub use omniroute::*;
pub use script_ai::*;
