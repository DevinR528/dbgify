// TODO remove
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_export]
macro_rules! bp {
    () => {};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
