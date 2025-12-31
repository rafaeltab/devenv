pub mod descriptor;

// Re-export commonly used types
pub use descriptor::{
    BranchDescriptor, CommitDescriptor, CreateContext, CreateError, Descriptor,
    DirectoryDescriptor, FileChange, GitRepoDescriptor, PathDescriptor, RemoteDescriptor,
    ResourceRegistry, TmuxSessionDescriptor, TmuxSessionInfo, TmuxSocket, WindowDescriptor,
};
