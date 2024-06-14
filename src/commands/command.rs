pub trait RafaeltabCommand<TArgs> {
    fn execute(&self, args: TArgs);
}
