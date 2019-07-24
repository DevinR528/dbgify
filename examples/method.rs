use dbgify::*;

struct Test(usize);

impl Test {
    #[dbgify]
    fn add_one(&mut self, x: usize) {
        bp!();
        self.0 += x;
    }
}
fn main() {
    let mut t = Test(1);
    t.add_one(1usize);
}
