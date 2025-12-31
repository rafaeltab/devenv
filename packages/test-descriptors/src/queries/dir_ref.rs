use crate::environment::TestEnvironment;
use std::path::{Path, PathBuf};

pub struct DirRef<'a> {
    pub(crate) path: PathBuf,
    pub(crate) env: &'a TestEnvironment,
}

impl<'a> DirRef<'a> {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn contains_file(&self, name: &str) -> bool {
        self.path.join(name).exists()
    }

    pub fn read_file(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.path.join(name)).ok()
    }
}
