use std::io;

pub trait Storage<TValue> {
    fn read(&self) -> &TValue;
    fn write(&mut self, value: &TValue) -> Result<(), io::Error>;
}

