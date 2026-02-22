use std::sync::Arc;

use shaku::Component;

use crate::di::ConfigPathProvider;
use crate::infrastructure::tmux_workspaces::tmux::connection::TmuxConnection;
use crate::storage::kinds::json_storage::JsonStorage;

#[derive(Component)]
#[shaku(interface = crate::domain::tmux_workspaces::repositories::tmux::session_repository::TmuxSessionRepository)]
pub struct TmuxRepository {
    #[shaku(inject)]
    pub connection: Arc<dyn TmuxConnection>,
    #[shaku(inject)]
    pub config_path_provider: Arc<dyn ConfigPathProvider>,
}

impl TmuxRepository {
    pub fn get_storage(&self) -> JsonStorage {
        let config_path = self.config_path_provider.path().to_string();
        JsonStorage::new(crate::storage::kinds::json_storage::JsonStorageParameters { config_path: config_path.clone() })
            .unwrap_or_else(|e| panic!(
                "Failed to load storage from '{}': {}. This should have been validated during container initialization.", 
                config_path, e
            ))
    }
}
