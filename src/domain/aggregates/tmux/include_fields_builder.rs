use super::{
    client::ClientIncludeFields, session::SessionIncludeFields, window::WindowIncludeFields,
};


#[allow(dead_code)]
pub struct IncludeFieldsBuilder {
    client: ClientIncludeFields,
    session: SessionIncludeFields,
    window: WindowIncludeFields,
}

#[allow(dead_code)]
impl IncludeFieldsBuilder {
    pub fn new() -> Self {
        IncludeFieldsBuilder {
            client: ClientIncludeFields { attached_to: None },
            session: SessionIncludeFields { windows: None },
            window: WindowIncludeFields { panes: None },
        }
    }

    pub fn with_attached_to(&self, attached_to: bool) -> Self {
        IncludeFieldsBuilder {
            client: ClientIncludeFields {
                attached_to: if attached_to {
                    Some(self.session.clone())
                } else {
                    None
                },
            },
            session: self.session.clone(),
            window: self.window.clone(),
        }
    }

    pub fn with_windows(&self, windows: bool) -> Self {
        let session = SessionIncludeFields {
            windows: if windows {
                Some(self.window.clone())
            } else {
                None
            },
        };
        IncludeFieldsBuilder {
            client: ClientIncludeFields {
                attached_to: self.client.attached_to.clone().map(|_| session.clone()),
            },
            session,
            window: self.window.clone(),
        }
    }

    pub fn with_panes(&self, panes: bool) -> Self {
        let window = WindowIncludeFields {
            panes: if panes { Some(()) } else { None },
        };

        let session = SessionIncludeFields {
            windows: self.session.windows.clone().map(|_| window.clone()),
        };
        IncludeFieldsBuilder {
            client: ClientIncludeFields {
                attached_to: self.client.attached_to.clone().map(|_| session.clone()),
            },
            session,
            window,
        }
    }

    pub fn build_client(&self) -> ClientIncludeFields {
        self.client.clone()
    }

    pub fn build_session(&self) -> SessionIncludeFields {
        self.session.clone()
    }

    pub fn build_window(&self) -> WindowIncludeFields {
        self.window.clone()
    }
}
