
#[allow(dead_code)]
pub enum TmuxListTarget<'a> {
    Workspace { id: &'a str },
    Session { id: &'a str },
    All,
}
