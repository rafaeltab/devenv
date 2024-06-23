#[derive(Clone)]
pub struct Workspace {
    /// A plaintext string that represents the unique identifier of this workspace
    pub id: String,
    /// A name that describes this workspace
    pub name: String,
    /// The path where this workspace resides
    pub path: String,
    /// Tags that provide additional information about this workspace
    pub tags: Vec<WorkspaceTag>,
    /// How important this workspace is, a higher value means this workspace is more important
    pub importance: i32,
}

#[derive(Clone)]
pub struct WorkspaceTag {
    /// The name of this tag
    pub name: String,
}
