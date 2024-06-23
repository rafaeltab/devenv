use crate::domain::aggregates::workspaces::workspace::Workspace;

pub trait WorkspaceRepository {
    fn get_workspaces(&self) -> Vec<Workspace>;
}
