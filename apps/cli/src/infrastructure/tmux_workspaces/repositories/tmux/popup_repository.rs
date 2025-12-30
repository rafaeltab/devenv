use crate::domain::tmux_workspaces::repositories::tmux::popup_repository::{
    PopupOptions, TmuxPopupRepository,
};

pub struct ImplPopupRepository;

impl TmuxPopupRepository for ImplPopupRepository {
    fn display_popup(&self, options: &PopupOptions) -> Result<(), String> {
        let mut args = vec!["display-popup".to_string()];

        // Target session
        args.push("-t".to_string());
        args.push(format!("{}:", options.target_session));

        // Width
        if let Some(ref width) = options.width {
            args.push("-w".to_string());
            args.push(width.clone());
        }

        // Height
        if let Some(ref height) = options.height {
            args.push("-h".to_string());
            args.push(height.clone());
        }

        // Title
        if let Some(ref title) = options.title {
            args.push("-T".to_string());
            args.push(title.clone());
        }

        // Command to execute
        args.push(options.command.clone());

        let output = std::process::Command::new("tmux")
            .args(&args)
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
