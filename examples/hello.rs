use rainbow_brackets::{RainbowBrackets, RainbowBracketsConfig};

fn main() {
    let rb = RainbowBracketsConfig::default();
    let colored = rb.colorize("fn foo(a: Vec<Vec<u8>>, b: (i32, i32)) {}");
    println!("{}", colored);

    #[derive(Debug, Default)]
    #[allow(unused)]
    struct Foo {
        x: Vec<Box<Foo>>,
        y: Option<(Box<Foo>, Box<Foo>)>,
    }

    let foo = Foo {
        x: vec![Box::new(Foo {
            x: vec![],
            y: Some((Box::new(Foo::default()), Box::new(Foo::default()))),
        })],
        y: Some((Box::new(Foo::default()), Box::new(Foo::default()))),
    };

    let colorized = (&foo).rainbow_brackets();
    println!("{:?}", colorized);
    println!("{:#?}", colorized);

    let full_colorized = foo.rainbow_brackets_with(RainbowBracketsConfig {
        colored_text: true,
        ..Default::default()
    });
    println!("{:?}", full_colorized);
    println!("{:#?}", full_colorized);
}
