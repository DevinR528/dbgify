use dbgify::*;

fn main() {
    #[dbgify]
    fn test(x: &mut String, y: usize) {
        bp!();
        match x {
            "hello" => x.push_str(" world"),
            _ => {},
        }
        bp!();
    }
    let mut x = "hello".to_string();
    test(&mut x, 10);
}
