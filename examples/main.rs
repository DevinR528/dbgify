#![feature(specialization)]
use dbgify::*;

fn main() {
    #[dbgify]
    fn test(x: &mut String, y: usize) {
        let z = 10;
        std::mem::drop(z);
        bp!();
        x.push_str(" world");
    }
    let mut x = "hello".to_string();
    test(&mut x, 10);
}
