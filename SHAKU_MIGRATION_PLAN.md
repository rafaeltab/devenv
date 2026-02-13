# Shaku Migration Plan

## Overview

This document outlines a plan to migrate the rafaeltab CLI application from manual dependency injection to using the **shaku** compile-time dependency injection container.

## Current State

The CLI currently uses manual dependency injection in `main.rs` (lines 220-427), which has several issues:

1. **Repetitive boilerplate** - Each command manually constructs its dependencies
2. **Deep nesting** - Repositories are nested inside other repositories (e.g., `ImplDescriptionRepository` contains `ImplWorkspaceRepository`)
3. **Lifetime complexity** - Managing lifetimes with `&` references is error-prone
4. **Test isolation difficulties** - Requires environment variables and `Box::leak` hacks (line 403)
5. **Code duplication** - Same repository construction repeated across multiple commands

### Example of Current Problem

```rust
// Lines 234-246 in main.rs - deeply nested manual construction
TmuxCommands::List(args) => TmuxListCommand.execute(TmuxListOptions {
    display: &*create_display(args),
    session_description_repository: &ImplDescriptionRepository {
        workspace_repository: &ImplWorkspaceRepository {
            workspace_storage: &storage,
        },
        session_repository: &TmuxRepository {
            tmux_storage: &storage,
            connection: &tmux_connection,
        },
        tmux_storage: &storage,
    },
}),
```

## Target Architecture with Shaku

### Benefits

1. **Centralized configuration** - All dependencies defined in one place
2. **Compile-time safety** - Type checking at compile time, no runtime failures
3. **Trait-based** - Clean separation between interfaces and implementations
4. **Lifecycle management** - Automatic Arc handling for shared dependencies
5. **Test-friendly** - Easy to swap implementations for testing
6. **Zero runtime overhead** - All resolution happens at compile time

## Migration Steps

### Phase 1: Add Dependencies

Add to `apps/cli/Cargo.toml`:

```toml
[dependencies]
shaku = "0.6"
```

### Phase 2: Annotate Traits and Implementations

#### 2.1 Update Repository Traits

Add `shaku::Interface` bound to existing traits:

```rust
// domain/tmux_workspaces/repositories/workspace/workspace_repository.rs
use shaku::Interface;

pub trait WorkspaceRepository: Interface {
    fn get_workspaces(&self) -> Vec<Workspace>;
    fn create_workspace(
        &self,
        name: String,
        tags: Vec<String>,
        root: String,
        id: String,
    ) -> Workspace;
}
```

Do this for all repository traits:

- `TmuxClientRepository`
- `TmuxSessionRepository`
- `SessionDescriptionRepository`
- `PopupRepository`
- etc.

#### 2.2 Annotate Implementations

```rust
// infrastructure/tmux_workspaces/repositories/workspace/workspace_repository.rs
use shaku::{Component, Interface};

#[derive(Component)]
#[shaku(interface = WorkspaceRepository)]
pub struct ImplWorkspaceRepository {
    #[shaku(inject)]
    workspace_storage: Arc<dyn WorkspaceStorage>,
}

impl WorkspaceRepository for ImplWorkspaceRepository {
    fn get_workspaces(&self) -> Vec<Workspace> {
        // Same implementation, uses self.workspace_storage
        self.workspace_storage
            .read()
            .iter()
            .map(|workspace| Workspace { /* ... */ })
            .collect()
    }
    // ...
}
```

#### 2.3 Storage Traits Also Need Interface

```rust
// storage/workspace.rs
use shaku::Interface;

pub trait WorkspaceStorage: Interface {
    fn read(&self) -> Vec<Workspace>;
    fn write(&self, workspaces: &[Workspace]) -> Result<(), StorageError>;
}
```

#### 2.4 Annotate Storage Implementations

```rust
// storage/kinds/json_storage.rs
use shaku::Component;

#[derive(Component)]
#[shaku(interface = WorkspaceStorage)]
pub struct JsonWorkspaceStorage {
    // Fields...
}
```

### Phase 3: Create Shaku Modules

#### 3.1 Define Production Module

Create `src/di/app_module.rs`:

```rust
use shaku::module;

module! {
    AppModule {
        components = [
            // Storage implementations
            JsonWorkspaceStorage,
            JsonTmuxStorage,
            JsonWorktreeStorage,

            // Repository implementations
            ImplWorkspaceRepository,
            TmuxRepository,
            ImplDescriptionRepository,
            ImplPopupRepository,

            // Display implementations
            PrettyDisplayProvider,
            JsonDisplayProvider,
        ],
        providers = [
            // For types that need custom construction
            TmuxConnectionProvider
        ]
    }
}

// Provider for TmuxConnection (needs environment-based construction)
#[derive(shaku::Provider)]
#[shaku(interface = TmuxConnection)]
struct TmuxConnectionProvider;

impl TmuxConnectionProvider {
    fn provide(&self) -> TmuxConnection {
        match std::env::var("RAFAELTAB_TMUX_SOCKET") {
            Ok(socket) => TmuxConnection::with_socket(socket),
            Err(_) => TmuxConnection::default(),
        }
    }
}
```

#### 3.2 Define Test Module

Create `src/di/test_module.rs`:

```rust
use shaku::module;

module! {
    TestModule {
        components = [
            MockWorkspaceStorage,
            MockTmuxStorage,
            ImplWorkspaceRepository,
            // Use real repositories with mock storage
        ],
        providers = []
    }
}
```

### Phase 4: Update Command Options

Current commands accept references with lifetimes:

```rust
pub struct TmuxListOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
    pub session_description_repository: &'a dyn SessionDescriptionRepository,
}
```

Update to use `Arc` (required for shaku):

```rust
use std::sync::Arc;

pub struct TmuxListOptions {
    pub display: Arc<dyn RafaeltabDisplay>,
    pub session_description_repository: Arc<dyn SessionDescriptionRepository>,
}
```

### Phase 5: Refactor Main.rs

#### 5.1 Create Container Builder

Create `src/di/container.rs`:

```rust
use shaku::HasComponent;
use std::sync::Arc;

pub struct AppContainer {
    module: Arc<dyn AppModule>,
}

impl AppContainer {
    pub fn new(config_path: Option<String>) -> Result<Self, io::Error> {
        let module = AppModule::builder()
            .with_component_parameters::<JsonStorageProvider>(
                JsonStorageProviderParameters { config: config_path }
            )
            .build();

        Ok(Self {
            module: Arc::new(module)
        })
    }

    pub fn workspace_repository(&self) -> Arc<dyn WorkspaceRepository> {
        self.module.resolve()
    }

    pub fn session_repository(&self) -> Arc<dyn TmuxSessionRepository> {
        self.module.resolve()
    }

    pub fn client_repository(&self) -> Arc<dyn TmuxClientRepository> {
        self.module.resolve()
    }

    pub fn description_repository(&self) -> Arc<dyn SessionDescriptionRepository> {
        self.module.resolve()
    }

    pub fn display(&self, args: &DisplayCommand) -> Arc<dyn RafaeltabDisplay> {
        match args {
            DisplayCommand { json: true, json_pretty: false, .. } => {
                self.module.resolve::<dyn JsonDisplay>()
            }
            DisplayCommand { json: true, json_pretty: true, .. } => {
                self.module.resolve::<dyn JsonPrettyDisplay>()
            }
            _ => self.module.resolve::<dyn PrettyDisplay>(),
        }
    }
}
```

#### 5.2 Simplified Main.rs

```rust
fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();

    // Single container creation - all dependencies resolved here
    let container = AppContainer::new(cli.config)?;

    match &cli.command {
        Some(Commands::Tmux(tmux_args)) => match &tmux_args.command {
            TmuxCommands::List(args) => {
                TmuxListCommand.execute(TmuxListOptions {
                    display: container.display(args),
                    session_description_repository: container.description_repository(),
                })
            }
            TmuxCommands::Start => {
                TmuxStartCommand.execute(TmuxStartOptions {
                    session_description_repository: container.description_repository(),
                    session_repository: container.session_repository(),
                    tmux_storage: container.tmux_storage(),
                })
            }
            // ... other commands simplified similarly
        },
        Some(Commands::Workspace(workspace_args)) => {
            // Much cleaner - no nested construction
            match &workspace_args.command {
                WorkspaceCommands::List(args) => {
                    ListWorkspacesCommand.execute(ListWorkspacesCommandArgs {
                        workspace_storage: container.workspace_repository(),
                        display: container.display(args),
                    })
                }
                // ...
            }
        }
        // ...
    }

    Ok(())
}
```

### Phase 6: Testing Improvements

#### 6.1 Test Container

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_container() -> Arc<dyn AppModule> {
        TestModule::builder()
            .with_component_parameters::<MockWorkspaceStorage>(
                MockWorkspaceStorageParameters {
                    data: test_data()
                }
            )
            .build()
    }

    #[test]
    fn test_list_workspaces() {
        let module = create_test_container();
        let repo: Arc<dyn WorkspaceRepository> = module.resolve();

        let workspaces = repo.get_workspaces();
        assert_eq!(workspaces.len(), 2);
    }
}
```

#### 6.2 No More Box::leak

The current code at line 403:

```rust
// TODO move to using DI so we don't have to do this guly magic
let storage_leaked = Box::leak(Box::new(storage));
```

Becomes:

```rust
// Clean and safe
let workspace_repository: Arc<dyn WorkspaceRepository> =
    container.workspace_repository();
let mut ctx = CommandCtx::new(workspace_repository)?;
```

## Detailed Implementation Order

### Week 1: Foundation

1. Add shaku dependency to Cargo.toml
2. Add `Interface` bound to all repository traits:
   - `WorkspaceRepository`
   - `TmuxClientRepository`
   - `TmuxSessionRepository`
   - `SessionDescriptionRepository`
   - `PopupRepository`
3. Add `Interface` bound to storage traits:
   - `WorkspaceStorage`
   - `TmuxStorage`
   - `WorktreeStorage`

### Week 2: Implementations

1. Annotate all repository implementations with `#[derive(Component)]`
2. Annotate all storage implementations with `#[derive(Component)]`
3. Update constructors to use `Arc` instead of references
4. Create `AppModule` definition

### Week 3: Commands

1. Update command option structs to use `Arc<dyn Trait>` instead of `&dyn Trait`
2. Update command implementations to work with `Arc`
3. Create `AppContainer` helper

### Week 4: Main.rs & Testing

1. Refactor main.rs to use container
2. Create `TestModule` for test scenarios
3. Migrate tests to use DI container
4. Remove `Box::leak` hacks
5. Remove environment variable-based test isolation hacks

## Code Examples

### Repository Trait Update

**Before:**

```rust
pub trait WorkspaceRepository {
    fn get_workspaces(&self) -> Vec<Workspace>;
}
```

**After:**

```rust
use shaku::Interface;

pub trait WorkspaceRepository: Interface {
    fn get_workspaces(&self) -> Vec<Workspace>;
}
```

### Repository Implementation Update

**Before:**

```rust
pub struct ImplWorkspaceRepository<'a, TWorkspaceStorage: WorkspaceStorage> {
    pub workspace_storage: &'a TWorkspaceStorage,
}

impl<TWorkspaceStorage> WorkspaceRepository for ImplWorkspaceRepository<'_, TWorkspaceStorage>
where
    TWorkspaceStorage: WorkspaceStorage,
{
    fn get_workspaces(&self) -> Vec<Workspace> {
        // Use self.workspace_storage
    }
}
```

**After:**

```rust
use shaku::Component;
use std::sync::Arc;

#[derive(Component)]
#[shaku(interface = WorkspaceRepository)]
pub struct ImplWorkspaceRepository {
    #[shaku(inject)]
    workspace_storage: Arc<dyn WorkspaceStorage>,
}

impl WorkspaceRepository for ImplWorkspaceRepository {
    fn get_workspaces(&self) -> Vec<Workspace> {
        // Use self.workspace_storage - same logic, different type
        self.workspace_storage.read().iter().map(|w| /* ... */).collect()
    }
}
```

### Command Options Update

**Before:**

```rust
pub struct TmuxListOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
    pub session_description_repository: &'a dyn SessionDescriptionRepository,
}
```

**After:**

```rust
use std::sync::Arc;

pub struct TmuxListOptions {
    pub display: Arc<dyn RafaeltabDisplay>,
    pub session_description_repository: Arc<dyn SessionDescriptionRepository>,
}
```

### Main.rs Update

**Before:**

```rust
let storage_provider = JsonStorageProvider::new(cli.config)?;
let storage = storage_provider.load()?;

let tmux_connection = match std::env::var("RAFAELTAB_TMUX_SOCKET") {
    Ok(socket) => TmuxConnection::with_socket(socket),
    Err(_) => TmuxConnection::default(),
};

match &cli.command {
    Some(Commands::Tmux(tmux_args)) => match &tmux_args.command {
        TmuxCommands::List(args) => TmuxListCommand.execute(TmuxListOptions {
            display: &*create_display(args),
            session_description_repository: &ImplDescriptionRepository {
                workspace_repository: &ImplWorkspaceRepository {
                    workspace_storage: &storage,
                },
                session_repository: &TmuxRepository {
                    tmux_storage: &storage,
                    connection: &tmux_connection,
                },
                tmux_storage: &storage,
            },
        }),
    }
}
```

**After:**

```rust
let container = AppContainer::new(cli.config)?;

match &cli.command {
    Some(Commands::Tmux(tmux_args)) => match &tmux_args.command {
        TmuxCommands::List(args) => TmuxListCommand.execute(TmuxListOptions {
            display: container.display(args),
            session_description_repository: container.description_repository(),
        }),
    }
}
```

## Testing Benefits

### Current Test Issues

1. Environment variables for test isolation (`RAFAELTAB_TMUX_SOCKET`)
2. `Box::leak` for lifetime issues
3. Hard to mock specific dependencies
4. Complex test setup

### With Shaku

1. **Clean test setup**:

   ```rust
   let module = TestModule::builder()
       .with_component_parameters::<MockWorkspaceStorage>(
           MockWorkspaceStorageParameters { data: test_workspaces() }
       )
       .build();
   ```

2. **Easy to swap implementations**:

   ```rust
   // Test with mock storage
   let test_module = TestModule::builder() /* ... */ .build();

   // Test with real storage but mock tmux
   let hybrid_module = HybridModule::builder() /* ... */ .build();
   ```

3. **No environment variables needed** - configure everything in the module

## Risks and Mitigations

### Risk 1: Arc Overhead

**Concern:** Moving from `&` references to `Arc` adds reference counting overhead.

**Mitigation:**

- This is minimal for a CLI tool (not a high-performance server)
- Shaku resolves once at startup, not per-operation
- Can use scoped dependencies for hot paths if needed

### Risk 2: Compile Time

**Concern:** Shaku macros might increase compile times.

**Mitigation:**

- Shaku is compile-time resolved but doesn't significantly impact compile times
- Module compilation is a one-time cost

### Risk 3: Learning Curve

**Concern:** Team needs to learn shaku patterns.

**Mitigation:**

- Shaku API is straightforward
- This migration plan provides clear examples
- Only affects dependency construction, not business logic

### Risk 4: Breaking Changes

**Concern:** Shaku is still evolving (v0.6).

**Mitigation:**

- Shaku is widely used and stable
- Changes would be in module definition, not business logic
- Easy to migrate if needed (all DI containers follow similar patterns)

## Success Criteria

1. ✅ Main.rs reduced from 445 lines to ~200 lines
2. ✅ No more nested repository construction
3. ✅ No more `Box::leak` or environment variable hacks in tests
4. ✅ Single source of truth for dependency configuration
5. ✅ All existing tests pass with new DI container
6. ✅ New tests can easily swap dependencies

## Alternative: Manual Builder Pattern

If shaku proves too complex, consider a manual builder pattern (see Option 5 in the original analysis). However, shaku provides better compile-time guarantees and requires less boilerplate.

## Conclusion

Migrating to shaku will:

- Reduce `main.rs` complexity by ~60%
- Eliminate manual dependency construction
- Improve testability without hacks
- Provide compile-time safety for dependency resolution
- Make the codebase more maintainable as it grows

The migration is estimated to take **4 weeks** working part-time on the refactoring.

---

## Additional Resources

- [Shaku Documentation](https://docs.rs/shaku/)
- [Shaku Examples](https://github.com/Azure/shaku/tree/master/shaku/examples)
- Rust DI Comparison: [https://www.lpalmieri.com/posts/a-journey-through-inversion-of-control-in-rust/]
