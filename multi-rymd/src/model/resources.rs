pub trait Resource {
    fn capacity(&self) -> i64;
    fn income(&self) -> i64;
    fn value(&self) -> i64;
    fn need(&self) -> i64;
}

pub struct Energy {
    pub current: i64
}

pub struct Metal {
    pub current: i64
}