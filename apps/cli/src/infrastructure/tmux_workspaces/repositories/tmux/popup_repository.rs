use shaku::Component;
use std::sync::Arc;

use crate::domain::tmux_workspaces::repositories::tmux::popup_repository::{
    PopupOptions, TmuxPopupRepository,
};
use crate::infrastructure::tmux_workspaces::tmux::connection::TmuxConnection;

#[derive(Component)]
#[shaku(interface = TmuxPopupRepository)]
pub struct ImplPopupRepository {
    #[shaku(inject)]
    pub connection: Arc<dyn TmuxConnection>,
}

impl TmuxPopupRepository for ImplPopupRepository {
    fn display_popup(&self, options: &PopupOptions) -> Result<(), String> {
        let mut cmd = self.connection.std_command();

        cmd.arg("display-popup");

        // Target session
        cmd.arg("-t");
        cmd.arg(format!("{}:", options.target_session));

        // Width
        if let Some(ref width) = options.width {
            cmd.arg("-w");
            cmd.arg(width);
        }

        // Height
        if let Some(ref height) = options.height {
            cmd.arg("-h");
            cmd.arg(height);
        }

        // Title
        if let Some(ref title) = options.title {
            cmd.arg("-T");
            cmd.arg(title);
        }

        // Command to execute
        cmd.arg(&options.command);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute tmux: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "tmux popup failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }
}
