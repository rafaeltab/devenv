use shaku::module;

use crate::{
    // Command components
    commands::{
        command_palette::CommandPaletteComponent,
        tmux::{
            list::TmuxListCommand, session_utils::ImplSessionUtilsService, start::TmuxStartCommand,
            switch::TmuxSwitchCommand,
        },
        workspaces::{
            add::WorkspaceAddCommand, current::CurrentWorkspaceCommand, find::FindWorkspaceCommand,
            find_tag::FindTagWorkspaceCommand, list::ListWorkspacesCommand,
            tmux::ListTmuxWorkspacesCommand,
        },
        worktree::{complete::WorktreeCompleteCommand, start::WorktreeStartCommand},
    },
    // Infrastructure components
    infrastructure::tmux_workspaces::{
        repositories::{
            tmux::{
                client_repository::ImplClientRepository,
                description_repository::ImplDescriptionRepository,
                pane_repository::ImplPaneRepository, popup_repository::ImplPopupRepository,
                session_repository::ImplSessionRepository, window_repository::ImplWindowRepository,
            },
            workspace::workspace_repository::ImplWorkspaceRepository,
        },
        tmux::{connection::TmuxConnection, session_detection::ImplSessionDetection},
    },
    // Storage wrapper components
    storage::kinds::json_storage::{JsonTmuxStorage, JsonWorkspaceStorage, JsonWorktreeStorage},
};

module! {
    pub AppModule {
        components = [
            // Storage wrappers (overridden at build time with real instances)
            JsonWorkspaceStorage,
            JsonTmuxStorage,
            JsonWorktreeStorage,

            // Infrastructure
            TmuxConnection,
            ImplSessionDetection,
            ImplPaneRepository,
            ImplWindowRepository,
            ImplSessionRepository,
            ImplClientRepository,
            ImplWorkspaceRepository,
            ImplPopupRepository,
            ImplDescriptionRepository,

            // Services
            ImplSessionUtilsService,

            // Commands
            TmuxListCommand,
            TmuxStartCommand,
            TmuxSwitchCommand,
            WorkspaceAddCommand,
            CurrentWorkspaceCommand,
            FindWorkspaceCommand,
            FindTagWorkspaceCommand,
            ListWorkspacesCommand,
            ListTmuxWorkspacesCommand,
            WorktreeStartCommand,
            WorktreeCompleteCommand,
            CommandPaletteComponent,
        ],
        providers = []
    }
}
