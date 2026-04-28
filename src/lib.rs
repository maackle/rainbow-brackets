//! Rainbow bracket colorization for terminal output.
//!
//! Formats a string with ANSI colors so that matching bracket pairs share a color,
//! and different nesting depths use different colors.
//!
//! # Example
//!
//! ```
//! use rainbow_brackets::{RainbowBracketsConfig};
//!
//! let rb = RainbowBracketsConfig::default();
//! let colored = rb.colorize("fn foo(a: Vec<u8>, b: (i32, i32)) {}");
//! println!("{}", colored);
//! ```

/// An ANSI terminal color.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    /// 256-color palette index (0–255).
    Ansi256(u8),
    /// 24-bit RGB color.
    Rgb(u8, u8, u8),
}

impl Color {
    fn ansi_fg(&self) -> String {
        match self {
            Color::Black => "\x1b[30m".into(),
            Color::Red => "\x1b[31m".into(),
            Color::Green => "\x1b[32m".into(),
            Color::Yellow => "\x1b[33m".into(),
            Color::Blue => "\x1b[34m".into(),
            Color::Magenta => "\x1b[35m".into(),
            Color::Cyan => "\x1b[36m".into(),
            Color::White => "\x1b[37m".into(),
            Color::BrightBlack => "\x1b[90m".into(),
            Color::BrightRed => "\x1b[91m".into(),
            Color::BrightGreen => "\x1b[92m".into(),
            Color::BrightYellow => "\x1b[93m".into(),
            Color::BrightBlue => "\x1b[94m".into(),
            Color::BrightMagenta => "\x1b[95m".into(),
            Color::BrightCyan => "\x1b[96m".into(),
            Color::BrightWhite => "\x1b[97m".into(),
            Color::Ansi256(n) => format!("\x1b[38;5;{n}m"),
            Color::Rgb(r, g, b) => format!("\x1b[38;2;{r};{g};{b}m"),
        }
    }
}

const RESET: &str = "\x1b[0m";

/// A bracket pair defined by its opening and closing characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BracketPair {
    pub open: char,
    pub close: char,
}

impl BracketPair {
    pub fn new(open: char, close: char) -> Self {
        Self { open, close }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Only colorize brackets.
    BracketsOnly,
    /// Color text between brackets the same as the outermost bracket.
    OuterText,
    /// Color text between brackets the same as the innermost bracket.
    InnerText,
}

impl Default for Mode {
    fn default() -> Self {
        Self::BracketsOnly
    }
}

/// Colorizes bracket pairs in a string by nesting depth.
///
/// Each depth level cycles through the configured `colors`. Non-bracket characters
/// are passed through unchanged unless `colored_text` is enabled, in which case
/// they are colored to match the innermost enclosing bracket pair.
#[derive(Clone)]
pub struct RainbowBracketsConfig {
    /// Colors used for each nesting depth, cycling if depth exceeds the list length.
    pub colors: Vec<Color>,
    /// Bracket pairs to colorize.
    pub pairs: Vec<BracketPair>,
    /// Mode to use for coloring text between brackets.
    pub mode: Mode,
}

impl Default for RainbowBracketsConfig {
    fn default() -> Self {
        Self {
            colors: vec![
                Color::BrightCyan,
                Color::BrightMagenta,
                Color::BrightYellow,
                Color::BrightBlack,
            ],
            pairs: vec![
                BracketPair::new('(', ')'),
                BracketPair::new('[', ']'),
                BracketPair::new('{', '}'),
                BracketPair::new('<', '>'),
                BracketPair::new('⟪', '⟫'),
            ],
            mode: Mode::BracketsOnly,
        }
    }
}

impl RainbowBracketsConfig {
    pub fn new(colors: Vec<Color>, pairs: Vec<BracketPair>, mode: Mode) -> Self {
        Self {
            colors,
            pairs,
            mode,
        }
    }

    /// Returns the colorized string with ANSI escape codes.
    ///
    /// Mismatched brackets (e.g. `(]`) are passed through without coloring.
    pub fn colorize(&self, input: &str) -> String {
        if self.colors.is_empty() {
            return input.to_string();
        }

        // Stack entries: (depth_index, expected_close_char)
        let mut stack: Vec<(usize, char)> = Vec::new();
        let mut depth: usize = 0;
        let mut out = String::with_capacity(input.len() + input.len() / 4);

        for ch in input.chars() {
            if let Some(pair) = self.pairs.iter().find(|p| p.open == ch) {
                // OuterText: the bracket lives in the outer zone (before opening), so
                // depth-0 brackets are uncolored and deeper ones use the outer zone's color.
                let open_colored = match self.mode {
                    Mode::OuterText => depth > 0,
                    _ => true,
                };
                if open_colored {
                    let idx = match self.mode {
                        Mode::OuterText => (depth - 1) % self.colors.len(),
                        _ => depth % self.colors.len(),
                    };
                    out.push_str(&self.colors[idx].ansi_fg());
                    out.push(ch);
                    out.push_str(RESET);
                } else {
                    out.push(ch);
                }
                stack.push((depth, pair.close));
                depth += 1;
            } else if self.pairs.iter().any(|p| p.close == ch) {
                // Check if this closes the innermost open bracket.
                if stack.last().map(|(_, c)| *c) == Some(ch) {
                    let (open_depth, _) = stack.pop().unwrap();
                    let (close_colored, idx) = match self.mode {
                        // InnerText: close bracket uses the inner depth (before restoring).
                        Mode::InnerText => (true, depth % self.colors.len()),
                        // OuterText: bracket lives in the outer zone (after closing);
                        // returning to depth 0 means no color.
                        Mode::OuterText if open_depth == 0 => (false, 0),
                        Mode::OuterText => (true, (open_depth - 1) % self.colors.len()),
                        Mode::BracketsOnly => (true, open_depth % self.colors.len()),
                    };
                    depth = open_depth;
                    if close_colored {
                        out.push_str(&self.colors[idx].ansi_fg());
                        out.push(ch);
                        out.push_str(RESET);
                    } else {
                        out.push(ch);
                    }
                } else {
                    // Mismatched — emit as-is.
                    out.push(ch);
                }
            } else {
                match self.mode {
                    Mode::BracketsOnly => out.push(ch),
                    Mode::OuterText if depth > 0 => {
                        let color = &self.colors[(depth - 1) % self.colors.len()];
                        out.push_str(&color.ansi_fg());
                        out.push(ch);
                        out.push_str(RESET);
                    }
                    Mode::InnerText if depth > 0 => {
                        let color = &self.colors[depth % self.colors.len()];
                        out.push_str(&color.ansi_fg());
                        out.push(ch);
                        out.push_str(RESET);
                    }
                    _ => out.push(ch),
                }
            }
        }

        out
    }
}

pub trait RainbowBrackets
where
    Self: Sized,
{
    fn rainbow_brackets(&self) -> RainbowBracketed<'_, Self> {
        RainbowBracketed {
            inner: self,
            config: RainbowBracketsConfig::default(),
        }
    }

    fn rainbow_brackets_with(&self, config: &RainbowBracketsConfig) -> RainbowBracketed<'_, Self> {
        RainbowBracketed {
            inner: self,
            config: config.clone(),
        }
    }
}

impl<T> RainbowBrackets for T where T: std::fmt::Debug {}

#[derive(Clone)]
pub struct RainbowBracketed<'a, T> {
    inner: &'a T,
    config: RainbowBracketsConfig,
}

impl<T> std::ops::Deref for RainbowBracketed<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::fmt::Display for RainbowBracketed<'_, T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.config.colorize(&self.inner.to_string()))
    }
}

impl<T> std::fmt::Debug for RainbowBracketed<'_, T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let plain = if f.alternate() {
            format!("{:#?}", self.inner)
        } else {
            format!("{:?}", self.inner)
        };
        write!(f, "{}", self.config.colorize(&plain))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        let rb = RainbowBracketsConfig::default();
        assert_eq!(rb.colorize(""), "");
    }

    #[test]
    fn no_brackets() {
        let rb = RainbowBracketsConfig::default();
        assert_eq!(rb.colorize("hello world"), "hello world");
    }

    #[test]
    fn single_pair() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
            ..Default::default()
        };
        let result = rb.colorize("(x)");
        assert!(result.contains("\x1b[91m") || result.contains("\x1b[31m"));
        assert!(result.contains(RESET));
    }

    #[test]
    fn nested_pairs_use_different_colors() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red, Color::Green],
            pairs: vec![BracketPair::new('(', ')')],
            ..Default::default()
        };
        let result = rb.colorize("((x))");
        // Depth 0 → Red, depth 1 → Green
        assert!(result.contains("\x1b[31m"));
        assert!(result.contains("\x1b[32m"));
    }

    #[test]
    fn colors_cycle() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
            ..Default::default()
        };
        // Three levels all use Red since there's only one color.
        let result = rb.colorize("(((x)))");
        assert_eq!(result.matches("\x1b[31m").count(), 6); // 3 opens + 3 closes
    }

    #[test]
    fn mismatched_bracket_passed_through() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')'), BracketPair::new('[', ']')],
            ..Default::default()
        };
        // `(]` — the `]` doesn't match `(`, should be emitted raw.
        let result = rb.colorize("(]");
        assert!(result.contains(']'));
        // The unmatched `(` was still colorized as an open bracket.
        assert!(result.contains("\x1b[31m"));
    }

    #[test]
    fn multiple_bracket_types() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red, Color::Green, Color::Blue],
            pairs: vec![
                BracketPair::new('(', ')'),
                BracketPair::new('[', ']'),
                BracketPair::new('{', '}'),
            ],
            ..Default::default()
        };
        let result = rb.colorize("({[]})");
        // Depth 0 → Red, depth 1 → Green, depth 2 → Blue
        assert!(result.contains("\x1b[31m")); // Red for outer ()
        assert!(result.contains("\x1b[32m")); // Green for {}
        assert!(result.contains("\x1b[34m")); // Blue for []
    }

    #[test]
    fn rgb_color() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Rgb(255, 128, 0)],
            pairs: vec![BracketPair::new('(', ')')],
            ..Default::default()
        };
        let result = rb.colorize("(x)");
        assert!(result.contains("\x1b[38;2;255;128;0m"));
    }

    // A close bracket that doesn't match the innermost open bracket is passed through as plain text.
    #[test]
    fn wrong_close_bracket_is_plain() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red, Color::Green],
            pairs: vec![BracketPair::new('(', ')'), BracketPair::new('[', ']')],
            ..Default::default()
        };
        // `([)]` — the `)` at position 2 doesn't match the innermost `[`, so it's raw.
        // The `]` at position 3 also doesn't match `(` (since `)` wasn't consumed), so it's raw too.
        let result = rb.colorize("([)]");
        // `)` is not preceded by any color escape — it's plain text.
        assert!(!result.contains("\x1b[31m)"));
        assert!(!result.contains("\x1b[32m)"));
        // Both characters appear in the output.
        assert!(result.contains(')'));
        assert!(result.contains(']'));
    }

    // An unclosed open bracket should still be colorized; the missing close is simply absent.
    #[test]
    fn unclosed_open_bracket() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
            ..Default::default()
        };
        let result = rb.colorize("(x");
        // The open bracket was colorized.
        assert!(result.contains("\x1b[31m"));
        // No close bracket in output at all.
        assert!(!result.contains(')'));
    }

    // A close bracket with no matching open is passed through as plain text.
    #[test]
    fn orphan_close_bracket() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
            ..Default::default()
        };
        let result = rb.colorize("x)y");
        // `)` appears but is not preceded by a color escape.
        assert_eq!(result, "x)y");
    }

    // Brackets not in the configured pairs must not be colorized.
    #[test]
    fn unconfigured_brackets_not_colored() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red],
            // Only round brackets configured; square and curly are not.
            pairs: vec![BracketPair::new('(', ')')],
            ..Default::default()
        };
        let result = rb.colorize("[x]{y}");
        // No ANSI codes at all — `[`, `]`, `{`, `}` are all plain.
        assert!(!result.contains('\x1b'));
        assert_eq!(result, "[x]{y}");
    }

    // --- RainbowBrackets trait / RainbowBracketed ---

    #[derive(Debug, Clone)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[derive(Debug, Clone)]
    #[allow(unused)]
    struct Tree {
        value: i32,
        children: Vec<Tree>,
    }

    // Regular debug output contains ANSI codes on brackets.
    #[test]
    fn trait_compact_debug_colored() {
        let data: Vec<Vec<i32>> = vec![vec![1, 2], vec![3, 4]];
        let wrapped = data.rainbow_brackets();
        let output = format!("{:?}", wrapped);
        // Nested brackets mean at least two distinct color depths.
        assert!(
            output.contains('\x1b'),
            "expected ANSI codes in compact debug"
        );
        // Original content is present.
        assert!(output.contains('1'));
        assert!(output.contains('4'));
    }

    // Pretty-printed debug output also contains ANSI codes.
    #[test]
    fn trait_pretty_debug_colored() {
        let tree = Tree {
            value: 1,
            children: vec![
                Tree {
                    value: 2,
                    children: vec![],
                },
                Tree {
                    value: 3,
                    children: vec![Tree {
                        value: 4,
                        children: vec![],
                    }],
                },
            ],
        };
        let wrapped = tree.rainbow_brackets();
        let output = format!("{:#?}", wrapped);
        assert!(
            output.contains('\x1b'),
            "expected ANSI codes in pretty debug"
        );
        // Multi-line pretty output should have newlines.
        assert!(output.contains('\n'));
        assert!(output.contains('4'));
    }

    // A custom struct with no brackets in its Debug output has no ANSI codes.
    #[test]
    fn trait_no_brackets_no_ansi() {
        #[derive(Debug)]
        #[allow(unused)]
        struct NoBrack {
            a: bool,
        }

        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red],
            // No bracket pairs configured.
            pairs: vec![],
            ..Default::default()
        };
        let wrapped = NoBrack { a: true }.rainbow_brackets_with(&rb);
        let output = format!("{:?}", wrapped);
        // NoBrack { a: true } — curly braces appear, but since pairs is empty
        // none of them should be colorized.
        assert!(!output.contains('\x1b'));
    }

    // Deref gives access to the inner value.
    #[test]
    fn trait_deref() {
        let pt = Point { x: 10, y: 20 };
        let wrapped = pt.rainbow_brackets();
        assert_eq!(wrapped.x, pt.x);
        assert_eq!(wrapped.y, pt.y);
    }

    // Display uses the inner value's Display impl (requires T: Display).
    #[test]
    fn trait_display_colored() {
        // String implements both Debug and Display.
        let s = String::from("hello (world)");
        let wrapped = s.rainbow_brackets();
        let display_out = format!("{}", wrapped);
        let debug_out = format!("{:?}", wrapped);
        // Display path colorizes the raw string value.
        assert!(display_out.contains('\x1b'));
        // Debug path wraps in quotes and colorizes brackets inside.
        assert!(debug_out.contains('\x1b'));
        assert!(debug_out.contains('"'));
    }

    // --- Mode tests ---

    // OuterText: depth-0 brackets are uncolored; only the text inside is colored.
    #[test]
    fn outer_text_basic() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
            mode: Mode::OuterText,
            ..Default::default()
        };
        // `(hi)` — `(` and `)` are at depth 0 (uncolored); `h` and `i` get Red.
        let result = rb.colorize("(hi)");
        assert_eq!(result.matches("\x1b[31m").count(), 2);
        assert!(result.starts_with('('));
        assert!(result.ends_with(')'));
    }

    // OuterText: text outside all brackets is never colored.
    #[test]
    fn outer_text_outside_brackets_uncolored() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
            mode: Mode::OuterText,
            ..Default::default()
        };
        let result = rb.colorize("a(b)c");
        assert!(result.starts_with('a'));
        assert!(result.ends_with('c'));
        assert!(result.contains("\x1b[31mb"));
    }

    // OuterText: nested brackets — brackets and text use the color of the zone they sit in.
    // The outermost `(` and `)` are uncolored; inner brackets take the surrounding zone's color.
    #[test]
    fn outer_text_nested_color_change() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red, Color::Green],
            pairs: vec![BracketPair::new('(', ')')],
            mode: Mode::OuterText,
            ..Default::default()
        };
        // `(a(b)c)`:
        //   outer `(` / `)` → uncolored (depth-0 zone)
        //   `a`, inner `(`, inner `)`, `c` → Red (depth-1 zone)
        //   `b` → Green (depth-2 zone)
        let result = rb.colorize("(a(b)c)");
        assert!(result.starts_with('(')); // outer `(` uncolored
        assert!(result.ends_with(')')); // outer `)` uncolored
        assert!(result.contains("\x1b[31ma")); // `a` → Red
        assert!(result.contains("\x1b[31m(")); // inner `(` → Red (in the Red zone)
        assert!(result.contains("\x1b[32mb")); // `b` → Green
        assert!(result.contains("\x1b[31m)")); // inner `)` → Red (returns to Red zone)
        assert!(result.contains("\x1b[31mc")); // `c` → Red
    }

    // BracketsOnly (default) leaves text chars uncolored.
    #[test]
    fn brackets_only_leaves_text_plain() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
            ..Default::default() // mode: BracketsOnly
        };
        let result = rb.colorize("(hi)");
        assert_eq!(result.matches("\x1b[31m").count(), 2);
        assert!(result.contains("hi"));
        assert!(!result.contains("\x1b[31mh"));
    }

    // InnerText: text inside brackets is colored with the inner (current) depth's color,
    // and the closing bracket shares that inner color.
    #[test]
    fn inner_text_basic() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red, Color::Green],
            pairs: vec![BracketPair::new('(', ')')],
            mode: Mode::InnerText,
            ..Default::default()
        };
        // `(hi)`: `(` at depth 0 → Red; text at depth 1 → Green; `)` uses inner depth 1 → Green.
        let result = rb.colorize("(hi)");
        assert!(result.contains("\x1b[31m(")); // open bracket → Red (depth 0)
        assert!(result.contains("\x1b[32mh")); // `h` → Green (depth 1)
        assert!(result.contains("\x1b[32mi")); // `i` → Green
        assert!(result.contains("\x1b[32m)")); // close bracket → Green (inner depth 1)
    }

    // InnerText: text outside all brackets is never colored.
    #[test]
    fn inner_text_outside_brackets_uncolored() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red, Color::Green],
            pairs: vec![BracketPair::new('(', ')')],
            mode: Mode::InnerText,
            ..Default::default()
        };
        let result = rb.colorize("a(b)c");
        assert!(result.starts_with('a'));
        assert!(result.ends_with('c'));
        assert!(result.contains("\x1b[32mb")); // `b` at depth 1 → Green
    }

    // InnerText: nested — text color matches the depth it sits at, and each close bracket
    // carries the inner color rather than the outer.
    #[test]
    fn inner_text_nested_color_change() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red, Color::Green, Color::Blue],
            pairs: vec![BracketPair::new('(', ')')],
            mode: Mode::InnerText,
            ..Default::default()
        };
        // `(a(b)c)`:
        //   `(` at depth 0 → Red
        //   `a` at depth 1 → Green
        //   `(` at depth 1 → Green
        //   `b` at depth 2 → Blue
        //   `)` closing depth 2→1 → Blue (inner)
        //   `c` at depth 1 → Green
        //   `)` closing depth 1→0 → Green (inner)
        let result = rb.colorize("(a(b)c)");
        assert!(result.contains("\x1b[31m(")); // outer `(` → Red
        assert!(result.contains("\x1b[32ma")); // `a` → Green
        assert!(result.contains("\x1b[34mb")); // `b` → Blue
        assert!(result.contains("\x1b[34m)")); // inner `)` → Blue
        assert!(result.contains("\x1b[32mc")); // `c` → Green
        assert!(result.contains("\x1b[32m)")); // outer `)` → Green
    }

    // A close character that is a close in *some* pair but not the one currently open
    // must not consume the stack entry.
    #[test]
    fn unconfigured_close_does_not_consume_stack() {
        let rb = RainbowBracketsConfig {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
            ..Default::default()
        };
        // `[` is not a configured open, so it's plain; `)` correctly closes `(`.
        let result = rb.colorize("([x])");
        // The outer `(` and `)` should be colorized.
        assert_eq!(result.matches("\x1b[31m").count(), 2);
        // `[` and `]` are plain.
        assert!(result.contains("[x]"));
    }
}
