use dbgify::*;

fn main() {
    #[dbgify]
    fn test(x: &mut String, y: usize) {
        bp!();
        x.push_str(" world");
        bp!()
    }
    let mut x = "hello".to_string();
    test(&mut x, 10);
}
