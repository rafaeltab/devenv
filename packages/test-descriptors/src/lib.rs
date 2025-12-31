pub mod descriptor;
pub mod environment;

// Re-export commonly used types
pub use descriptor::{
    BranchDescriptor, CommitDescriptor, CreateContext, CreateError, Descriptor,
    DirectoryDescriptor, FileChange, GitRepoDescriptor, PathDescriptor, RemoteDescriptor,
    ResourceRegistry, TmuxSessionDescriptor, TmuxSessionInfo, TmuxSocket, WindowDescriptor,
};
pub use environment::TestEnvironment;
