use std::io;

pub trait Storage<TValue> {
    fn read(&self) -> TValue;
    fn write(&self, value: &TValue) -> Result<(), io::Error>;
}
