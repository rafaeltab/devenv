use super::context::CreateContext;
use super::error::CreateError;
use super::traits::{Descriptor, PathDescriptor};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct DirectoryDescriptor {
    name: String,
}

impl DirectoryDescriptor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Descriptor for DirectoryDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError> {
        let path = self.path(context);

        // Create directory and all parent directories
        fs::create_dir_all(&path)?;

        // Register the directory in context
        context.register_resource(self.name.clone(), path);

        Ok(())
    }
}

impl PathDescriptor for DirectoryDescriptor {
    fn path(&self, context: &CreateContext) -> PathBuf {
        context.root_path().join(&self.name)
    }
}
