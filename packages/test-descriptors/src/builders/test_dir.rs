use super::dir::DirBuilder;
use crate::descriptor::Descriptor;
use std::path::PathBuf;

pub struct TestDirBuilder {
    parent_path: PathBuf,
    descriptors: Vec<Box<dyn Descriptor>>,
}

impl TestDirBuilder {
    pub(crate) fn new(parent_path: PathBuf) -> Self {
        Self {
            parent_path,
            descriptors: Vec::new(),
        }
    }

    pub fn dir<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut DirBuilder),
    {
        let mut builder = DirBuilder::new(name, self.parent_path.clone());
        f(&mut builder);
        self.descriptors.push(Box::new(builder.build()));
    }

    pub(crate) fn into_descriptors(self) -> Vec<Box<dyn Descriptor>> {
        self.descriptors
    }
}
