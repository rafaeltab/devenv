use super::session::{SessionIncludeFields, TmuxSession};

#[derive(Debug, Clone)]
pub struct TmuxClient {
    pub name: String,
    pub attached_to: Option<TmuxSession>,
    pub include_fields: ClientIncludeFields,
}

#[derive(Clone, Debug)]
pub struct ClientIncludeFields {
    pub attached_to: Option<SessionIncludeFields>,
}
