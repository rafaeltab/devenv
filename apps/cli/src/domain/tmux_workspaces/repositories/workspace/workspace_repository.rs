use crate::domain::tmux_workspaces::aggregates::workspaces::workspace::Workspace;

pub trait WorkspaceRepository {
    fn get_workspaces(&self) -> Vec<Workspace>;
    fn create_workspace(
        &self,
        name: String,
        tags: Vec<String>,
        root: String,
        id: String,
    ) -> Workspace;
}
