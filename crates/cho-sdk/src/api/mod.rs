//! API namespace helpers.

pub mod resource;
pub mod specs;

pub use resource::ResourceApi;
pub use specs::{RESOURCES, ResourceCapabilities, ResourceSpec, by_name};
