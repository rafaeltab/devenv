use super::git::GitBuilder;
use crate::descriptor::{CreateContext, CreateError, Descriptor};
use std::path::PathBuf;

pub struct DirBuilder {
    name: String,
    parent_path: PathBuf,
    children: Vec<Box<dyn Descriptor>>,
}

impl DirBuilder {
    pub(crate) fn new(name: &str, parent_path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            parent_path,
            children: Vec::new(),
        }
    }

    /// Get the full path this directory will be created at
    fn our_path(&self) -> PathBuf {
        self.parent_path.join(&self.name)
    }

    pub fn dir<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut DirBuilder),
    {
        let mut builder = DirBuilder::new(name, self.our_path());
        f(&mut builder);
        self.children.push(Box::new(builder.build()));
    }

    pub fn git<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut GitBuilder),
    {
        let mut builder = GitBuilder::new(name, self.our_path());
        f(&mut builder);
        self.children.push(Box::new(builder.build()));
    }

    pub(crate) fn build(self) -> DirDescriptor {
        DirDescriptor {
            name: self.name,
            parent_path: self.parent_path,
            children: self.children,
        }
    }
}

/// Hierarchical directory descriptor that can contain children
#[derive(Debug)]
pub struct DirDescriptor {
    name: String,
    parent_path: PathBuf,
    children: Vec<Box<dyn Descriptor>>,
}

impl Descriptor for DirDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError> {
        let path = self.parent_path.join(&self.name);

        // Create this directory
        std::fs::create_dir_all(&path)?;

        // Register in context
        context
            .registry()
            .borrow_mut()
            .register_dir(self.name.clone(), path.clone());

        // Create all children
        for child in &self.children {
            child.create(context)?;
        }

        Ok(())
    }
}
