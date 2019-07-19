use dbgify::*;

fn main() {
    #[dbgify]
    fn test(x: &mut String) {
        bp!();
        x.push_str(" world");
    }

    let mut x = "hello".to_string();

    test(&mut x);
}
