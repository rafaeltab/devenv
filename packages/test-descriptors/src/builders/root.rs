use super::test_dir::TestDirBuilder;
use crate::descriptor::Descriptor;
use crate::environment::TestEnvironment;
use std::path::Path;

pub struct RootBuilder<'a> {
    env: &'a mut TestEnvironment,
}

impl<'a> RootBuilder<'a> {
    pub(crate) fn new(env: &'a mut TestEnvironment) -> Self {
        Self { env }
    }

    /// Get the root path of the test environment
    pub fn root_path(&self) -> &Path {
        self.env.root_path()
    }

    /// Add a descriptor to be created
    ///
    /// This is useful for extension traits to add custom descriptors
    pub fn add_descriptor<D: Descriptor + 'static>(&mut self, descriptor: D) {
        self.env.add_boxed_descriptor(Box::new(descriptor));
    }

    pub fn test_dir<F>(&mut self, f: F)
    where
        F: FnOnce(&mut TestDirBuilder),
    {
        let mut builder = TestDirBuilder::new(self.env.root_path().to_path_buf());
        f(&mut builder);

        // Add all descriptors to environment
        for descriptor in builder.into_descriptors() {
            self.env.add_boxed_descriptor(descriptor);
        }
    }
}
