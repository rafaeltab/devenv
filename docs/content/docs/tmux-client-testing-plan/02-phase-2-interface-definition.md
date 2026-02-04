---
title: Phase 2 Interface Definition
---

# Phase 2: Interface Definition

**Goal:** Define all public interfaces with `todo!()` implementations so tests compile but fail.

**Prerequisite:** Phase 1 complete (test files exist)

## Files to Create

### 1. Traits (`packages/test-descriptors/src/testers/traits.rs`)

```rust
pub trait CommandTester {
    fn run(&self, cmd: &Command) -> CommandResult;
}

pub trait TuiTester {
    type Asserter: TuiAsserter;
    fn run(&self, cmd: &Command) -> Self::Asserter;
}
```

### 2. TuiAsserter Trait (`packages/test-descriptors/src/testers/tui_asserter.rs`)

```rust
/// Trait defining the interface for TUI test assertions and interactions.
///
/// All TUI testers return types that implement this trait, allowing tests
/// to be written generically against any TUI execution backend.
pub trait TuiAsserter {
    // Lifecycle
    fn wait_for_settle(&mut self);
    fn wait_for_settle_ms(&mut self, timeout_ms: u64, max_wait_ms: u64);
    fn wait_ms(&mut self, ms: u64);
    fn expect_completion(&mut self) -> i32;
    fn expect_exit_code(&mut self, expected: i32);

    // Input
    fn type_text(&mut self, text: &str);
    fn press_key(&mut self, key: Key);
    fn send_keys(&mut self, keys: &[Key]);

    // Queries
    fn find_text(&self, text: &str) -> TextMatch;
    fn find_all_text(&self, text: &str) -> Vec<TextMatch>;
    fn screen(&self) -> String;

    // Debug
    fn dump_screen(&self);
}
```

### 3. Command (`packages/test-descriptors/src/testers/command.rs`)

```rust
pub struct Command {
    program: String,
    args: Vec<String>,
    envs: HashMap<String, String>,
    cwd: Option<PathBuf>,
}

impl Command {
    pub fn new(program: impl Into<String>) -> Self;
    pub fn args(self, args: &[&str]) -> Self;
    pub fn arg(self, arg: impl Into<String>) -> Self;
    pub fn env(self, key: impl Into<String>, value: impl Into<String>) -> Self;
    pub fn envs(self, envs: HashMap<String, String>) -> Self;
    pub fn cwd(self, path: impl AsRef<Path>) -> Self;

    // Internal methods for testers
    pub(crate) fn program(&self) -> &str;
    pub(crate) fn get_args(&self) -> &[String];
    pub(crate) fn get_envs(&self) -> &HashMap<String, String>;
    pub(crate) fn get_cwd(&self) -> Option<&PathBuf>;
}
```

### 4. CommandResult (`packages/test-descriptors/src/testers/command.rs`)

```rust
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}
```

### 5. TextMatch (`packages/test-descriptors/src/testers/text_match.rs`)

```rust
pub struct TextMatch {
    pub fg: ColorAssertion,
    pub bg: ColorAssertion,
    // Internal fields
}

impl TextMatch {
    pub fn position(&self) -> Option<(u16, u16)>;
    pub fn assert_visible(&self);
    pub fn assert_not_visible(&self);
}
```

### 6. Color Types (`packages/test-descriptors/src/testers/color.rs`)

```rust
pub struct ColorAssertion {
    // Internal fields
}

impl ColorAssertion {
    pub fn assert(&self, matcher: ColorMatcher);
    pub fn exact(&self, r: u8, g: u8, b: u8);
}

pub enum ColorMatcher {
    Grayscale,
    RedIsh,
    GreenIsh,
    BlueIsh,
    YellowIsh,
    CyanIsh,
    MagentaIsh,
    Hue { min: f32, max: f32 },
}

impl ColorMatcher {
    pub fn matches(&self, r: u8, g: u8, b: u8) -> bool;
}
```

### 7. Keys (`packages/test-descriptors/src/testers/keys.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Tab,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Ctrl,
    Alt,
    Shift,
    Super,
}
```

### 8. Tester Structs

#### CmdTester (`packages/test-descriptors/src/testers/cmd/cmd_tester.rs`)

```rust
pub struct CmdTester;

impl CmdTester {
    pub(crate) fn new() -> Self;
}

impl CommandTester for CmdTester {
    fn run(&self, cmd: &Command) -> CommandResult;
}
```

#### PtyTester (`packages/test-descriptors/src/testers/pty/pty_tester.rs`)

```rust
pub struct PtyTester {
    rows: u16,
    cols: u16,
    settle_timeout_ms: u64,
}

impl PtyTester {
    pub(crate) fn new() -> Self;
    pub fn terminal_size(self, rows: u16, cols: u16) -> Self;
    pub fn settle_timeout(self, ms: u64) -> Self;
}

impl TuiTester for PtyTester {
    type Asserter = PtyAsserter;
    fn run(&self, cmd: &Command) -> Self::Asserter;
}
```

#### PtyAsserter (`packages/test-descriptors/src/testers/pty/pty_asserter.rs`)

```rust
pub struct PtyAsserter {
    // Internal: PTY handle, terminal buffer, etc.
}

impl TuiAsserter for PtyAsserter {
    // All trait methods implemented
}
```

#### TmuxClientCmdTester (`packages/test-descriptors/src/testers/tmux_client_cmd/tmux_client_cmd_tester.rs`)

```rust
pub struct TmuxClientCmdTester<'a> {
    client: &'a TmuxClientHandle,
    socket: &'a TmuxSocket,
}

impl<'a> TmuxClientCmdTester<'a> {
    pub(crate) fn new(client: &'a TmuxClientHandle, socket: &'a TmuxSocket) -> Self;
}

impl CommandTester for TmuxClientCmdTester<'_> {
    fn run(&self, cmd: &Command) -> CommandResult;
}
```

#### TmuxClientPtyTester (`packages/test-descriptors/src/testers/tmux_client_pty/tmux_client_pty_tester.rs`)

```rust
pub struct TmuxClientPtyTester<'a> {
    client: &'a TmuxClientHandle,
    socket: &'a TmuxSocket,
    settle_timeout_ms: u64,
}

impl<'a> TmuxClientPtyTester<'a> {
    pub(crate) fn new(client: &'a TmuxClientHandle, socket: &'a TmuxSocket) -> Self;
    pub fn settle_timeout(self, ms: u64) -> Self;
}

impl TuiTester for TmuxClientPtyTester<'_> {
    type Asserter = CapturePaneAsserter;
    fn run(&self, cmd: &Command) -> Self::Asserter;
}
```

#### CapturePaneAsserter (`packages/test-descriptors/src/testers/tmux_client_pty/capture_pane_asserter.rs`)

```rust
pub struct CapturePaneAsserter {
    // Internal: socket, session name, terminal buffer, etc.
}

impl TuiAsserter for CapturePaneAsserter {
    // All trait methods implemented
}
```

#### TmuxFullClientTester (`packages/test-descriptors/src/testers/tmux_full_client/tmux_full_client_tester.rs`)

```rust
pub struct TmuxFullClientTester<'a> {
    client: &'a TmuxClientHandle,
    settle_timeout_ms: u64,
}

impl<'a> TmuxFullClientTester<'a> {
    pub(crate) fn new(client: &'a TmuxClientHandle) -> Self;
    pub fn settle_timeout(self, ms: u64) -> Self;
}

impl TuiTester for TmuxFullClientTester<'_> {
    type Asserter = FullClientAsserter;
    fn run(&self, cmd: &Command) -> Self::Asserter;
}
```

#### FullClientAsserter (`packages/test-descriptors/src/testers/tmux_full_client/full_client_asserter.rs`)

```rust
pub struct FullClientAsserter {
    // Internal: PTY reader/writer, terminal buffer, etc.
}

impl TuiAsserter for FullClientAsserter {
    // All trait methods implemented
}
```

### 9. TesterFactory (`packages/test-descriptors/src/testers/factory.rs`)

```rust
pub struct TesterFactory<'a> {
    env: &'a TestEnvironment,
}

impl<'a> TesterFactory<'a> {
    pub(crate) fn new(env: &'a TestEnvironment) -> Self;

    pub fn cmd(&self) -> CmdTester;
    pub fn pty(&self) -> PtyTester;
    pub fn tmux_client_cmd(&self) -> TmuxClientCmdTester;
    pub fn tmux_client_pty(&self) -> TmuxClientPtyTester;
    pub fn tmux_full_client(&self) -> TmuxFullClientTester;
}
```

### 10. TmuxClientHandle (`packages/test-descriptors/src/descriptor/tmux_client.rs`)

```rust
pub struct TmuxClientHandle {
    session_name: String,
    pty_pair: PtyPair,
    child: Box<dyn Child + Send + Sync>,
    pty_size: (u16, u16),
}

impl TmuxClientHandle {
    pub fn current_session(&self) -> String;
    pub(crate) fn pty_size(&self) -> (u16, u16);
    pub(crate) fn session_name(&self) -> &str;
}
```

### 11. TmuxClientDescriptor (`packages/test-descriptors/src/descriptor/tmux_client.rs`)

```rust
pub struct TmuxClientDescriptor {
    session_name: String,
    pty_rows: u16,
    pty_cols: u16,
}

impl Descriptor for TmuxClientDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError>;
}
```

### 12. ClientBuilder (`packages/test-descriptors/src/builders/tmux.rs`)

Add to existing file:

```rust
pub struct ClientBuilder {
    pty_rows: u16,
    pty_cols: u16,
}

impl ClientBuilder {
    pub fn pty_size(&mut self, rows: u16, cols: u16);
}
```

### 13. SessionBuilder Extension

Modify existing `SessionBuilder`:

```rust
impl SessionBuilder {
    // Existing methods...

    pub fn with_client<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ClientBuilder);
}
```

### 14. TestEnvironment Extensions

Modify existing `TestEnvironment`:

```rust
impl TestEnvironment {
    // Existing methods...

    pub fn testers(&self) -> TesterFactory;
    pub fn has_tmux_client(&self) -> bool;
    pub fn tmux_client(&self) -> Option<&TmuxClientHandle>;
}
```

## Module Structure

```rust
// packages/test-descriptors/src/testers/mod.rs

// Trait definitions at top level
mod traits;
mod tui_asserter;
mod command;
mod text_match;
mod color;
mod keys;

// Implementation folders
mod cmd;
mod pty;
mod tmux_client_cmd;
mod tmux_client_pty;
mod tmux_full_client;
mod internal;
mod factory;

// Public re-exports
pub use traits::{CommandTester, TuiTester};
pub use tui_asserter::TuiAsserter;
pub use command::{Command, CommandResult};
pub use text_match::TextMatch;
pub use color::{ColorAssertion, ColorMatcher};
pub use keys::Key;
pub use factory::TesterFactory;

// Re-export testers (users get them from factory, but may need types)
pub use cmd::CmdTester;
pub use pty::{PtyTester, PtyAsserter};
pub use tmux_client_cmd::TmuxClientCmdTester;
pub use tmux_client_pty::{TmuxClientPtyTester, CapturePaneAsserter};
pub use tmux_full_client::{TmuxFullClientTester, FullClientAsserter};
```

```rust
// packages/test-descriptors/src/lib.rs
// Add:
pub mod testers;
pub use testers::{
    Command, CommandResult, CommandTester,
    TuiAsserter, TuiTester, TextMatch,
    ColorAssertion, ColorMatcher, Key,
    TesterFactory,
};
```

## Implementation Notes

All method bodies should be:

```rust
pub fn method_name(&self) -> ReturnType {
    todo!("Phase N: Implement method_name")
}
```

Where N is the phase number where this method will be implemented.

## Deliverables

1. All struct/enum definitions
2. All trait definitions
3. All method signatures with `todo!()` bodies
4. Module declarations and re-exports
5. Tests should now compile but all fail with "not yet implemented" panics
