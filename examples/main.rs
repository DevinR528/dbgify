use dbgify::*;

fn main() {
    #[dbgify]
    fn test(x: &mut String) {
        let _y = 0;
        bp!();
        x.push_str(" world");
    }
    let mut x = "hello".to_string();
    test(&mut x);
}
