pub mod builders;
pub mod descriptor;
pub mod environment;
pub mod queries;
pub mod testers;

// Re-export commonly used types
pub use builders::{
    BranchBuilder, ClientBuilder, CommitBuilder, DirBuilder, GitBuilder, RootBuilder,
    SessionBuilder, TestDirBuilder, WorktreeBuilder,
};
pub use descriptor::{
    BranchDescriptor, CommitDescriptor, CreateContext, CreateError, Descriptor,
    DirectoryDescriptor, FileChange, GitRepoDescriptor, PathDescriptor, RemoteDescriptor,
    ResourceRegistry, TmuxClientDescriptor, TmuxClientHandle, TmuxSessionDescriptor,
    TmuxSessionInfo, TmuxSocket, WindowDescriptor,
};
pub use environment::TestEnvironment;
pub use queries::{DirRef, GitRepoRef, ShellOutput, TmuxSessionRef, WorktreeRef};

// Re-export testers module types
pub use testers::{
    CapturePaneAsserter, CmdTester, ColorAssertion, ColorMatcher, Command, CommandResult,
    CommandTester, FullClientAsserter, Key, Modifier, PtyAsserter, PtyTester, TesterFactory,
    TextMatch, TmuxClientCmdTester, TmuxClientPtyTester, TmuxFullClientTester, TuiAsserter,
    TuiTester,
};
