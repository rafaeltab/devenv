pub mod branch;
pub mod commit;
pub mod context;
pub mod directory;
pub mod error;
pub mod git_repo;
pub mod registry;
pub mod remote;
pub mod traits;

pub use branch::BranchDescriptor;
pub use commit::{CommitDescriptor, FileChange};
pub use context::CreateContext;
pub use directory::DirectoryDescriptor;
pub use error::CreateError;
pub use git_repo::GitRepoDescriptor;
pub use registry::{ResourceRegistry, TmuxSessionInfo};
pub use remote::RemoteDescriptor;
pub use traits::{Descriptor, PathDescriptor};
