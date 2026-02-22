# Command Palette Migration Plan: Dependency Injection Refactoring

This document outlines the migration plan for refactoring the command palette to use dependency injection with Shaku, similar to the CLI commands.

## 📋 Current Architecture

### Current Command Execution Flow

```
main.rs
  ↓
CommandRegistry::new()
  ↓
registry.register(AddWorkspaceCommand::new())
  ↓
CommandPalette::new(registry)
  ↓
CommandCtx::new(workspace_repository)
  ↓
palette.run(&mut ctx) [passes ctx to commands]
  ↓
command.run(&mut ctx) [commands use ctx.workspace_repo()]
```

### Current Components

1. **CommandCtx** (`apps/cli/src/commands/command_ctx.rs`)

   - Currently holds: `PickerCtx` + `WorkspaceRepository`
   - Provides: `workspace_repo()`, `select()`, `input()`, `confirm()`, etc.
   - **Problem**: Hard to extend with new repositories/services

2. **Command Trait** (`apps/cli/src/commands/command.rs`)

   ```rust
   pub trait Command {
       fn name(&self) -> &str;
       fn description(&self) -> &str;
       fn run(&self, ctx: &mut CommandCtx);
   }
   ```

3. **CommandRegistry** (`apps/cli/src/commands/registry.rs`)

   - Stores commands as `Vec<Rc<dyn Command>>`
   - Currently created and populated in `main.rs`

4. **AddWorkspaceCommand** (`apps/cli/src/commands/builtin/add_workspace.rs`)
   - Only palette command currently (besides test commands)
   - Accesses workspace repository via `ctx.workspace_repo()`

## 🎯 Goals

1. **Extensibility**: Make it easy to add commands that need different repositories/services
2. **Consistency**: Align command palette architecture with CLI command DI pattern
3. **Maintainability**: Centralize dependency resolution in the DI container
4. **Flexibility**: Support commands with varied dependency requirements

## 🔄 Proposed Architecture

### New Command Execution Flow

```
main.rs
  ↓
container.command_palette() [resolves from DI]
  ↓
CommandPalette (with pre-registered commands from DI)
  ↓
CommandCtx::new() [no repositories, just PickerCtx]
  ↓
palette.run(&mut ctx)
  ↓
command.run(&mut ctx) [commands use self.repository fields]
```

### Key Changes

#### 1. Commands Become Stateful (Hold Dependencies)

**Before:**

```rust
#[derive(Debug)]
pub struct AddWorkspaceCommand;

impl Command for AddWorkspaceCommand {
    fn run(&self, ctx: &mut CommandCtx) {
        let workspaces = ctx.workspace_repo().get_workspaces();
    }
}
```

**After:**

```rust
pub struct AddWorkspaceCommand {
    workspace_repository: Arc<dyn WorkspaceRepository>,
}

impl Command for AddWorkspaceCommand {
    fn run(&self, ctx: &mut CommandCtx) {
        // Use self.workspace_repository instead of ctx
        let workspaces = self.workspace_repository.get_workspaces();
    }
}
```

#### 2. CommandCtx Becomes Repository-Agnostic

**Before:**

```rust
pub struct CommandCtx {
    picker_ctx: PickerCtx,
    workspace_repo: Arc<dyn WorkspaceRepository>,  // ❌ Hardcoded
}

impl CommandCtx {
    pub fn new(workspace_repo: Arc<dyn WorkspaceRepository>) -> io::Result<Self> {
        Ok(Self {
            picker_ctx: PickerCtx::new()?,
            workspace_repo,
        })
    }

    pub fn workspace_repo(&self) -> &dyn WorkspaceRepository {
        self.workspace_repo.as_ref()
    }
}
```

**After:**

```rust
pub struct CommandCtx {
    picker_ctx: PickerCtx,  // ✅ Only UI concerns
}

impl CommandCtx {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            picker_ctx: PickerCtx::new()?,
        })
    }

    // workspace_repo() method removed
    // Commands access their own injected dependencies
}
```

#### 3. Commands Registered in DI Container

**Before (in main.rs):**

```rust
let mut registry = CommandRegistry::new();
registry.register(AddWorkspaceCommand::new());

let palette = CommandPalette::new(registry);
let mut ctx = CommandCtx::new(workspace_repository)?;
palette.run(&mut ctx);
```

**After (in DI container):**

```rust
// In apps/cli/src/di/mod.rs
impl AppContainer {
    pub fn new(config_path: Option<String>) -> Result<Self, std::io::Error> {
        // ... existing setup ...

        // Create commands with dependencies
        let add_workspace_cmd = Rc::new(AddWorkspaceCommand {
            workspace_repository: module.resolve(),
        });

        // Create registry and register commands
        let mut registry = CommandRegistry::new();
        registry.register_rc(add_workspace_cmd);

        // Create command palette with pre-configured registry
        let command_palette = Arc::new(CommandPalette::new(registry));

        Ok(Self {
            // ... existing fields ...
            command_palette,
        })
    }
}

// In main.rs - much simpler!
let palette = container.command_palette();
let mut ctx = CommandCtx::new()?;
palette.run(&mut ctx);
```

## 📝 Migration Steps

### Step 1: Update CommandRegistry API

Add a method to register `Rc<dyn Command>` directly:

```rust
// In apps/cli/src/commands/registry.rs
impl CommandRegistry {
    /// Register a command from an Rc (for DI-injected commands)
    pub fn register_rc(&mut self, command: Rc<dyn Command>) -> &mut Self {
        self.commands.push(command);
        self
    }
}
```

### Step 2: Refactor CommandCtx

Remove repository dependencies:

```rust
// In apps/cli/src/commands/command_ctx.rs
pub struct CommandCtx {
    picker_ctx: PickerCtx,
    // Remove: workspace_repo: Arc<dyn WorkspaceRepository>,
}

impl CommandCtx {
    pub fn new() -> io::Result<Self> {
        let picker_ctx = PickerCtx::new()?;
        Ok(Self { picker_ctx })
    }

    // Remove workspace_repo() method
}
```

### Step 3: Update AddWorkspaceCommand

Add injected dependencies:

```rust
// In apps/cli/src/commands/builtin/add_workspace.rs
pub struct AddWorkspaceCommand {
    workspace_repository: Arc<dyn WorkspaceRepository>,
}

impl AddWorkspaceCommand {
    pub fn new(workspace_repository: Arc<dyn WorkspaceRepository>) -> Self {
        Self { workspace_repository }
    }
}

impl Command for AddWorkspaceCommand {
    fn run(&self, ctx: &mut CommandCtx) {
        // Change: ctx.workspace_repo().get_workspaces()
        // To: self.workspace_repository.get_workspaces()
        let workspaces = self.workspace_repository.get_workspaces();
        // ... rest of implementation
    }
}
```

### Step 4: Register Commands in DI Container

Add command palette setup to `AppContainer`:

```rust
// In apps/cli/src/di/mod.rs
use crate::commands::{
    builtin::AddWorkspaceCommand,
    registry::CommandRegistry,
    CommandPalette,
};

pub struct AppContainer {
    // ... existing fields ...
    command_palette: Arc<CommandPalette>,
}

impl AppContainer {
    pub fn new(config_path: Option<String>) -> Result<Self, std::io::Error> {
        // ... existing setup ...

        // Create command palette commands with dependencies
        let add_workspace_cmd = Rc::new(AddWorkspaceCommand::new(
            module.resolve()  // WorkspaceRepository
        ));

        // Create registry and register commands
        let mut registry = CommandRegistry::new();
        registry.register_rc(add_workspace_cmd);

        // Optionally register test commands
        #[cfg(test)]
        if std::env::var("TEST_MODE").is_ok() {
            registry.register(TestPickerCommand::new());
            registry.register(TestTextInputCommand::new());
            registry.register(TestTextInputSuggestionsCommand::new());
            registry.register(TestConfirmCommand::new());
        }

        // Create command palette
        let command_palette = Arc::new(CommandPalette::new(registry));

        Ok(Self {
            // ... existing fields ...
            command_palette,
        })
    }

    pub fn command_palette(&self) -> Arc<CommandPalette> {
        Arc::clone(&self.command_palette)
    }
}
```

### Step 5: Simplify main.rs

Update command palette invocation:

```rust
// In apps/cli/src/main.rs
Some(Commands::CommandPalette(palette_args)) => {
    use crate::commands::Command;

    match &palette_args.command {
        CommandPaletteCommands::Show => {
            // Get palette from DI container (already configured!)
            let palette = container.command_palette();

            if palette.registry().is_empty() {
                println!("No commands available");
            } else {
                // Create simple context (no repositories needed)
                let mut ctx = crate::commands::CommandCtx::new()
                    .expect("Failed to create command context");

                // Run palette
                palette.run(&mut ctx);
            }
        }
    }
}
```

### Step 6: Update Tests

Update any tests that create `CommandCtx`:

```rust
// Before:
let ctx = CommandCtx::new(workspace_repository)?;

// After:
let ctx = CommandCtx::new()?;
```

Update any tests that create commands:

```rust
// Before:
let cmd = AddWorkspaceCommand::new();

// After:
let cmd = AddWorkspaceCommand::new(workspace_repository);
```

## 🔮 Future Enhancements

### Example: Command Requiring Multiple Dependencies

Once this pattern is established, adding complex commands is straightforward:

```rust
pub struct StartWorktreeCommand {
    workspace_repository: Arc<dyn WorkspaceRepository>,
    session_repository: Arc<dyn TmuxSessionRepository>,
    client_repository: Arc<dyn TmuxClientRepository>,
    popup_repository: Arc<dyn TmuxPopupRepository>,
}

impl StartWorktreeCommand {
    pub fn new(
        workspace_repository: Arc<dyn WorkspaceRepository>,
        session_repository: Arc<dyn TmuxSessionRepository>,
        client_repository: Arc<dyn TmuxClientRepository>,
        popup_repository: Arc<dyn TmuxPopupRepository>,
    ) -> Self {
        Self {
            workspace_repository,
            session_repository,
            client_repository,
            popup_repository,
        }
    }
}

impl Command for StartWorktreeCommand {
    fn name(&self) -> &str { "Start Worktree" }

    fn description(&self) -> &str {
        "Create and start a new git worktree"
    }

    fn run(&self, ctx: &mut CommandCtx) {
        // All dependencies available via self
        let branch_name = ctx.input("Branch name")?;

        // Use injected repositories
        let workspace = self.workspace_repository.get_current();
        self.session_repository.create_session(&branch_name);
        self.client_repository.switch_to(&branch_name);
        // ... etc
    }
}

// Registration in DI:
let start_worktree_cmd = Rc::new(StartWorktreeCommand::new(
    module.resolve(),  // workspace_repository
    module.resolve(),  // session_repository
    module.resolve(),  // client_repository
    module.resolve(),  // popup_repository
));
registry.register_rc(start_worktree_cmd);
```

### Alternative: Generic Command Context

If you want to keep CommandCtx extensible for common dependencies:

```rust
pub struct CommandCtx {
    picker_ctx: PickerCtx,
    // Instead of hardcoded repositories, use a type-safe registry
    dependencies: DependencyMap,
}

impl CommandCtx {
    // Type-safe dependency access
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.dependencies.get::<T>()
    }
}

// Commands can still use ctx, but it's more flexible:
fn run(&self, ctx: &mut CommandCtx) {
    if let Some(repo) = ctx.get::<Arc<dyn WorkspaceRepository>>() {
        // Use repo
    }
}
```

However, the **recommended approach** is the simpler one: commands hold their own dependencies, and `CommandCtx` only provides UI interaction methods.

## 🧪 Testing Strategy

1. **Unit tests**: Test commands in isolation with mock repositories
2. **Integration tests**: Test command palette with real DI container
3. **Regression tests**: Ensure existing AddWorkspaceCommand behavior unchanged

## ✅ Success Criteria

A migration is considered complete when:

1. ✅ `CommandCtx` no longer holds repository dependencies
2. ✅ Commands hold their required dependencies as fields
3. ✅ Commands are registered in `AppContainer` with dependencies injected
4. ✅ `main.rs` retrieves command palette from container
5. ✅ All existing command palette functionality works unchanged
6. ✅ Easy to add new commands with different dependency requirements
7. ✅ Tests pass

## 🎓 Benefits

### Before DI Migration

**To add a command needing `TmuxSessionRepository`:**

1. Update `CommandCtx` to include `TmuxSessionRepository`
2. Update `CommandCtx::new()` to accept the new parameter
3. Update all call sites of `CommandCtx::new()` (breaking change!)
4. Add getter method to `CommandCtx`
5. Create command

**5 steps, 3 files changed, breaking change to API**

### After DI Migration

**To add a command needing `TmuxSessionRepository`:**

1. Create command with required dependencies in constructor
2. Register in `AppContainer` with dependencies resolved

**2 steps, 2 files changed, no breaking changes**

## 📚 Reference Implementation

See the CLI command migration in `COMMAND_MIGRATION_PLAN.md` for similar patterns:

- Dependency injection via constructor
- Registration in `AppContainer`
- Usage in `main.rs`

## 🚀 Migration Order

Recommended order (one step at a time):

1. **Step 1**: Add `register_rc()` method to `CommandRegistry`
2. **Step 2-3**: Refactor `CommandCtx` and `AddWorkspaceCommand` together
3. **Step 4**: Add command palette to `AppContainer`
4. **Step 5**: Simplify `main.rs`
5. **Step 6**: Update tests
6. **Verify**: Run tests and manually test command palette

## ⚠️ Potential Challenges

1. **Rc vs Arc**: Commands use `Rc<dyn Command>` while repositories use `Arc`

   - **Solution**: Keep using `Rc` for commands (single-threaded), `Arc` for repositories (may be shared)

2. **Test commands**: Test commands don't need dependencies

   - **Solution**: Keep using `.register()` for simple commands, use `.register_rc()` for DI commands

3. **Backward compatibility**: Changing `CommandCtx` API
   - **Solution**: This is internal API, no external consumers. Safe to change.

## 🎉 Conclusion

This migration will:

- Make the command palette more extensible
- Align with the CLI command DI pattern
- Make it trivial to add new commands with arbitrary dependencies
- Keep `CommandCtx` focused on UI concerns only
- Centralize all dependency wiring in the DI container

The result is a more maintainable, testable, and extensible command palette system!
