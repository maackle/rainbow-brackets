use rainbow_brackets::RainbowBrackets;

fn main() {
    let rb = RainbowBrackets::default();
    let colored = rb.colorize("fn foo(a: Vec<Vec<u8>>, b: (i32, i32)) {}");
    println!("{}", colored);
}
