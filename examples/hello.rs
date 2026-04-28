use rainbow_brackets::{Mode, RainbowBrackets, RainbowBracketsConfig};

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

    let colorized = foo.rainbow_brackets();
    println!("{:?}", colorized);
    println!("{:#?}", colorized);

    println!(
        "{:?}",
        foo.rainbow_brackets_with(&RainbowBracketsConfig {
            mode: Mode::InnerText,
            ..Default::default()
        })
    );
    println!(
        "{:?}",
        foo.rainbow_brackets_with(&RainbowBracketsConfig {
            mode: Mode::OuterText,
            ..Default::default()
        })
    );
}
