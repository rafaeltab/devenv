pub mod builders;
pub mod descriptor;
pub mod environment;
pub mod queries;

// Re-export commonly used types
pub use builders::{DirBuilder, RootBuilder, TestDirBuilder};
pub use descriptor::{
    BranchDescriptor, CommitDescriptor, CreateContext, CreateError, Descriptor,
    DirectoryDescriptor, FileChange, GitRepoDescriptor, PathDescriptor, RemoteDescriptor,
    ResourceRegistry, TmuxSessionDescriptor, TmuxSessionInfo, TmuxSocket, WindowDescriptor,
};
pub use environment::TestEnvironment;
pub use queries::DirRef;
