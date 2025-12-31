pub mod descriptor;

// Re-export commonly used types
pub use descriptor::{
    CreateContext, CreateError, Descriptor, DirectoryDescriptor, GitRepoDescriptor, PathDescriptor,
    ResourceRegistry, TmuxSessionInfo,
};
