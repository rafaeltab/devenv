use super::context::CreateContext;
use super::error::CreateError;
use std::path::PathBuf;

/// Base trait for all descriptors
pub trait Descriptor: std::fmt::Debug {
    /// Materialize this descriptor into the filesystem/environment
    fn create(&self, context: &CreateContext) -> Result<(), CreateError>;
}

/// Trait for descriptors that represent a path
pub trait PathDescriptor: Descriptor {
    /// Get the path this descriptor will create
    fn path(&self, context: &CreateContext) -> PathBuf;
}
