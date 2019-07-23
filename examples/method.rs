use dbgify::*;

struct Test;

impl Test {
    #[dbgify]
    fn test(&self, x: &mut usize) -> usize {
        bp!();
        x += 1;
        x
    }
}
fn main() {
    let x = Test::test(1usize);
    println!("{}", x)
}
