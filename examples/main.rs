#[macro_use]
use dbgify::*;

#[dbgify]
fn main() {
    
    fn test(x: &mut String) {
        bp!();
        x.push_str(" world");
    }
    
    let mut x = "hello".to_string();

    test(&mut x);

    println!("{}", x)
}
