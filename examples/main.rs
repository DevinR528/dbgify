#![feature(specialization)]
use dbgify::*;

fn main() {
    #[dbgify]
    fn test(x: &mut Vec<String>) {
        let _y = 10;
        bp!();
        x.push(" world".into());
    }
    let mut x = vec!["hello".to_string()];
    test(&mut x);
}
