# Command Migration Plan: Dependency Injection Refactoring

This document outlines the migration plan for refactoring all CLI commands to use dependency injection with Shaku.

## ✅ Completed

- [x] Infrastructure setup (DisplayFactory, CommandError)
- [x] Updated RafaeltabCommand trait to return Result
- [x] **TmuxSwitchCommand** - Pilot implementation
- [x] **TmuxListCommand** - Simple command with display factory pattern
- [x] **TmuxStartCommand** - Simple command, no display, no args
- [x] **ListWorkspacesCommand** - Workspace command with display factory, no args
- [x] **WorkspaceAddCommand** - Command with display and CLI arguments
- [x] **WorktreeStartCommand** - Worktree command with CLI arguments, no display
- [x] **WorktreeCompleteCommand** - Worktree command with CLI arguments, no display
- [x] **CurrentWorkspaceCommand** - Converted from function to struct with display factory
- [x] **FindWorkspaceCommand** - Converted from function to struct with display and CLI argument
- [x] **FindTagWorkspaceCommand** - Converted from function to struct with display and CLI argument
- [x] **ListTmuxWorkspacesCommand** - Converted from function to struct with display factory

## 📋 Commands To Migrate

### Tmux Commands

- [x] **TmuxListCommand** (`apps/cli/src/commands/tmux/list.rs`)
  - Has display (use DisplayFactory)
  - No CLI arguments
- [x] **TmuxStartCommand** (`apps/cli/src/commands/tmux/start.rs`)
  - No display
  - No CLI arguments

### Workspace Commands (Struct-based)

- [x] **ListWorkspacesCommand** (`apps/cli/src/commands/workspaces/list.rs`)
  - Has display (use DisplayFactory)
  - No CLI arguments
- [x] **WorkspaceAddCommand** (`apps/cli/src/commands/workspaces/add.rs`)
  - Has display (use DisplayFactory)
  - Has CLI arguments: `interactive`, `name`, `tags`, `path`

### Worktree Commands

- [x] **WorktreeStartCommand** (`apps/cli/src/commands/worktree/start.rs`)
  - No display
  - Has CLI arguments: `branch_name`, `force`, `yes`
- [x] **WorktreeCompleteCommand** (`apps/cli/src/commands/worktree/complete.rs`)
  - No display
  - Has CLI arguments: `branch_name`, `force`, `yes`

### Workspace Commands (Function-based)

- [x] **get_current_workspace** (`apps/cli/src/commands/workspaces/current.rs`)
  - Has display (use DisplayFactory)
  - No CLI arguments
  - Convert from function to struct
- [x] **find_workspace_cmd** (`apps/cli/src/commands/workspaces/find.rs`)
  - Has display (use DisplayFactory)
  - Has CLI argument: `id` (String)
  - Convert from function to struct
- [x] **find_tag_workspace** (`apps/cli/src/commands/workspaces/find_tag.rs`)
  - Has display (use DisplayFactory)
  - Has CLI argument: `tag` (String)
  - Convert from function to struct
- [x] **list_tmux_workspaces** (`apps/cli/src/commands/workspaces/tmux.rs`)
  - Has display (use DisplayFactory)
  - No CLI arguments
  - Convert from function to struct

---

## 🔧 Migration Pattern

Follow this pattern for each command migration:

### Step 1: Update the Command Structure

**Before (stateless):**

```rust
#[derive(Default)]
pub struct MyCommand;

pub struct MyCommandOptions {
    pub some_repository: Arc<dyn SomeRepository>,
    pub display: Arc<dyn RafaeltabDisplay>,
    pub some_arg: String,
}
```

**After (with injected dependencies):**

```rust
// Runtime options - CLI arguments only
pub struct MyCommandRuntimeOptions {
    pub some_arg: String,  // Only CLI args here
}

// Command with injected dependencies
pub struct MyCommand {
    pub some_repository: Arc<dyn SomeRepository>,
    pub display_factory: Arc<dyn DisplayFactory>,  // If command uses display
    pub config_path_provider: Arc<dyn ConfigPathProvider>,  // If needed
}
```

### Step 2: Update the execute() Implementation

**For commands with display:**

```rust
impl RafaeltabCommand<MyCommandRuntimeOptions> for MyCommand {
    fn execute(&self, options: MyCommandRuntimeOptions) -> Result<(), CommandError> {
        // Create display from factory based on runtime options
        let display = self.display_factory.create_display(
            options.json,
            options.json_pretty
        );

        // Use injected dependencies via self
        let data = self.some_repository.get_data();
        display.display(&data);

        Ok(())
    }
}
```

**For commands without display:**

```rust
impl RafaeltabCommand<MyCommandRuntimeOptions> for MyCommand {
    fn execute(&self, options: MyCommandRuntimeOptions) -> Result<(), CommandError> {
        // Use injected dependencies via self
        let data = self.some_repository.get_data();

        // Use runtime options
        if options.some_arg {
            // ...
        }

        Ok(())
    }
}
```

### Step 3: Register in DI Container

Add to `apps/cli/src/di/mod.rs`:

```rust
impl AppContainer {
    pub fn new(config_path: Option<String>) -> Result<Self, std::io::Error> {
        // ... existing code ...

        // Add command instantiation
        let my_command = Arc::new(MyCommand {
            some_repository: module.resolve(),
            display_factory: module.resolve(),  // If needed
            config_path_provider: module.resolve(),  // If needed
        });

        Ok(Self {
            // ... existing fields ...
            my_command,  // Add to struct
        })
    }

    // Add resolver method
    pub fn my_command(&self) -> Arc<MyCommand> {
        Arc::clone(&self.my_command)
    }
}

// Update struct definition
pub struct AppContainer {
    // ... existing fields ...
    my_command: Arc<MyCommand>,
}
```

### Step 4: Update main.rs

**Before:**

```rust
MyCommand.execute(MyCommandOptions {
    some_repository: container.some_repository(),
    display: create_display_arc(args),
    some_arg: args.some_arg.clone(),
})
```

**After:**

```rust
let command = container.my_command();
command.execute(MyCommandRuntimeOptions {
    some_arg: args.some_arg.clone(),
    json: args.json,
    json_pretty: args.json_pretty,
})
.expect("Failed to execute my command");
```

### Step 5: Update imports in main.rs

Remove the old Options import, add RuntimeOptions:

```rust
// Before:
use commands::my_module::{MyCommand, MyCommandOptions};

// After:
use commands::my_module::MyCommandRuntimeOptions;
```

The command itself will be resolved from the container, not imported directly.

---

## 🧪 Testing Protocol

**CRITICAL:** After migrating each command, you MUST:

1. **Build the project:**

   ```bash
   cargo build --manifest-path apps/cli/Cargo.toml
   ```

2. **Run all tests:**

   ```bash
   turbo run test
   ```

3. **Verify all tests pass**

   - All existing tests must continue to pass
   - If tests fail, fix the migration before proceeding to the next command

4. **Manual smoke test (if applicable):**
   - Run the command manually to ensure it works as expected
   - Example: `cargo run --manifest-path apps/cli/Cargo.toml -- tmux list`

---

## 📝 Special Cases

### Function-Style Commands

For commands currently implemented as functions (e.g., `get_current_workspace`):

1. **Convert to struct pattern** following the same structure as other commands
2. **Move the function logic** into the `execute()` method
3. **Create RuntimeOptions** struct if there are CLI arguments
4. **Remove the function** and replace with struct in exports

**Example:**

Before:

```rust
pub fn get_current_workspace(
    workspace_repository: Arc<dyn WorkspaceRepository>,
    options: CurrentWorkspaceOptions,
) {
    // function logic
}
```

After:

```rust
pub struct CurrentWorkspaceRuntimeOptions {
    pub json: bool,
    pub json_pretty: bool,
}

pub struct CurrentWorkspaceCommand {
    pub workspace_repository: Arc<dyn WorkspaceRepository>,
    pub display_factory: Arc<dyn DisplayFactory>,
}

impl RafaeltabCommand<CurrentWorkspaceRuntimeOptions> for CurrentWorkspaceCommand {
    fn execute(&self, options: CurrentWorkspaceRuntimeOptions) -> Result<(), CommandError> {
        let display = self.display_factory.create_display(options.json, options.json_pretty);
        // original function logic here
        Ok(())
    }
}
```

### Commands with DisplayCommand Args

For commands that receive a `DisplayCommand` from clap (with `json` and `json_pretty` fields):

**In RuntimeOptions:**

```rust
pub struct MyCommandRuntimeOptions {
    pub json: bool,
    pub json_pretty: bool,
    // ... other CLI args
}
```

**In main.rs:**

```rust
command.execute(MyCommandRuntimeOptions {
    json: args.display_command.json,
    json_pretty: args.display_command.json_pretty,
    // ... other args
})
```

---

## 🎯 Success Criteria

A command migration is considered complete when:

1. ✅ Command struct has injected dependencies as fields
2. ✅ RuntimeOptions struct contains only CLI arguments
3. ✅ execute() method returns `Result<(), CommandError>`
4. ✅ Command is registered in DI container
5. ✅ main.rs uses container to resolve command
6. ✅ Project builds without errors
7. ✅ **All tests pass with `turbo run test`**
8. ✅ Manual smoke test confirms functionality

---

## 📚 Reference Implementation

See `TmuxSwitchCommand` in `apps/cli/src/commands/tmux/switch.rs` for a complete reference implementation.

Key files:

- Command: `apps/cli/src/commands/tmux/switch.rs:32-78`
- DI Registration: `apps/cli/src/di/mod.rs:73-78` and `127-129`
- main.rs Usage: `apps/cli/src/main.rs:232-236`

---

## 🚀 Recommended Order

Migrate in this order to build confidence with the pattern:

1. **TmuxListCommand** - Simple, has display factory pattern
2. **TmuxStartCommand** - Simple, no display, no args
3. **WorktreeStartCommand** - Has runtime args, no display
4. **ListWorkspacesCommand** - Has display, no args
5. **WorkspaceAddCommand** - Has both display and args
6. **WorktreeCompleteCommand** - Has args, no display
7. **get_current_workspace** - Convert function to struct
8. **find_workspace_cmd** - Convert function with arg
9. **find_tag_workspace** - Convert function with arg
10. **list_tmux_workspaces** - Convert function

---

## ⚠️ Common Pitfalls

1. **Forgetting to update imports in main.rs** - Remove old Options imports
2. **Not handling Result return type** - Add `.expect()` in main.rs
3. **Wrong ConfigPathProvider method** - Use `.path()` not `.get_config_path()`
4. **DisplayFactory fields** - Commands with display need `display_factory`, not `display`
5. **Missing fields in AppContainer** - Don't forget to add command fields to the struct
6. **Not running tests** - ALWAYS run `turbo run test` after each migration

---

## 📞 Questions?

If you encounter issues during migration:

1. Check the reference implementation (TmuxSwitchCommand)
2. Verify all fields are properly injected in DI container
3. Ensure RuntimeOptions only contains CLI args
4. Make sure tests pass before moving to next command

Good luck with the migration! 🎉
