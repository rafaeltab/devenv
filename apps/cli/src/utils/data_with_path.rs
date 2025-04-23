pub struct DataWithPath<T> {
    pub data: T,
    pub path: String,
}

impl<T> DataWithPath<T> {
    pub fn new(data: T, path: String) -> Self {
        DataWithPath { data, path }
    }
}
