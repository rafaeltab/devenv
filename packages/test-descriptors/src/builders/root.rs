use super::test_dir::TestDirBuilder;
use crate::environment::TestEnvironment;

pub struct RootBuilder<'a> {
    env: &'a mut TestEnvironment,
}

impl<'a> RootBuilder<'a> {
    pub(crate) fn new(env: &'a mut TestEnvironment) -> Self {
        Self { env }
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
