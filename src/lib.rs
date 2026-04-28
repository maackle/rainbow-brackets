//! Rainbow bracket colorization for terminal output.
//!
//! Formats a string with ANSI colors so that matching bracket pairs share a color,
//! and different nesting depths use different colors.
//!
//! # Example
//!
//! ```
//! use rainbow_brackets::{RainbowBrackets, Color};
//!
//! let rb = RainbowBrackets::default();
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

/// Colorizes bracket pairs in a string by nesting depth.
///
/// Each depth level cycles through the configured `colors`. Non-bracket characters
/// are passed through unchanged with the ANSI reset applied after each bracket.
#[derive(Debug, Clone)]
pub struct RainbowBrackets {
    /// Colors used for each nesting depth, cycling if depth exceeds the list length.
    pub colors: Vec<Color>,
    /// Bracket pairs to colorize.
    pub pairs: Vec<BracketPair>,
}

impl Default for RainbowBrackets {
    fn default() -> Self {
        Self {
            colors: vec![
                Color::BrightYellow,
                Color::BrightMagenta,
                Color::BrightCyan,
                Color::BrightGreen,
                Color::Yellow,
                Color::Magenta,
                Color::Cyan,
                Color::Green,
            ],
            pairs: vec![
                BracketPair::new('(', ')'),
                BracketPair::new('[', ']'),
                BracketPair::new('{', '}'),
                BracketPair::new('<', '>'),
            ],
        }
    }
}

impl RainbowBrackets {
    pub fn new(colors: Vec<Color>, pairs: Vec<BracketPair>) -> Self {
        Self { colors, pairs }
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
                let color = &self.colors[depth % self.colors.len()];
                out.push_str(&color.ansi_fg());
                out.push(ch);
                out.push_str(RESET);
                stack.push((depth, pair.close));
                depth += 1;
            } else if self.pairs.iter().any(|p| p.close == ch) {
                // Check if this closes the innermost open bracket.
                if stack.last().map(|(_, c)| *c) == Some(ch) {
                    let (open_depth, _) = stack.pop().unwrap();
                    depth = open_depth;
                    let color = &self.colors[depth % self.colors.len()];
                    out.push_str(&color.ansi_fg());
                    out.push(ch);
                    out.push_str(RESET);
                } else {
                    // Mismatched — emit as-is.
                    out.push(ch);
                }
            } else {
                out.push(ch);
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        let rb = RainbowBrackets::default();
        assert_eq!(rb.colorize(""), "");
    }

    #[test]
    fn no_brackets() {
        let rb = RainbowBrackets::default();
        assert_eq!(rb.colorize("hello world"), "hello world");
    }

    #[test]
    fn single_pair() {
        let rb = RainbowBrackets {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
        };
        let result = rb.colorize("(x)");
        assert!(result.contains("\x1b[91m") || result.contains("\x1b[31m"));
        assert!(result.contains(RESET));
    }

    #[test]
    fn nested_pairs_use_different_colors() {
        let rb = RainbowBrackets {
            colors: vec![Color::Red, Color::Green],
            pairs: vec![BracketPair::new('(', ')')],
        };
        let result = rb.colorize("((x))");
        // Depth 0 → Red, depth 1 → Green
        assert!(result.contains("\x1b[31m"));
        assert!(result.contains("\x1b[32m"));
    }

    #[test]
    fn colors_cycle() {
        let rb = RainbowBrackets {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
        };
        // Three levels all use Red since there's only one color.
        let result = rb.colorize("(((x)))");
        assert_eq!(result.matches("\x1b[31m").count(), 6); // 3 opens + 3 closes
    }

    #[test]
    fn mismatched_bracket_passed_through() {
        let rb = RainbowBrackets {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')'), BracketPair::new('[', ']')],
        };
        // `(]` — the `]` doesn't match `(`, should be emitted raw.
        let result = rb.colorize("(]");
        assert!(result.contains(']'));
        // The unmatched `(` was still colorized as an open bracket.
        assert!(result.contains("\x1b[31m"));
    }

    #[test]
    fn multiple_bracket_types() {
        let rb = RainbowBrackets {
            colors: vec![Color::Red, Color::Green, Color::Blue],
            pairs: vec![
                BracketPair::new('(', ')'),
                BracketPair::new('[', ']'),
                BracketPair::new('{', '}'),
            ],
        };
        let result = rb.colorize("({[]})");
        // Depth 0 → Red, depth 1 → Green, depth 2 → Blue
        assert!(result.contains("\x1b[31m")); // Red for outer ()
        assert!(result.contains("\x1b[32m")); // Green for {}
        assert!(result.contains("\x1b[34m")); // Blue for []
    }

    #[test]
    fn rgb_color() {
        let rb = RainbowBrackets {
            colors: vec![Color::Rgb(255, 128, 0)],
            pairs: vec![BracketPair::new('(', ')')],
        };
        let result = rb.colorize("(x)");
        assert!(result.contains("\x1b[38;2;255;128;0m"));
    }

    // A close bracket that doesn't match the innermost open bracket is passed through as plain text.
    #[test]
    fn wrong_close_bracket_is_plain() {
        let rb = RainbowBrackets {
            colors: vec![Color::Red, Color::Green],
            pairs: vec![BracketPair::new('(', ')'), BracketPair::new('[', ']')],
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
        let rb = RainbowBrackets {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
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
        let rb = RainbowBrackets {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
        };
        let result = rb.colorize("x)y");
        // `)` appears but is not preceded by a color escape.
        assert_eq!(result, "x)y");
    }

    // Brackets not in the configured pairs must not be colorized.
    #[test]
    fn unconfigured_brackets_not_colored() {
        let rb = RainbowBrackets {
            colors: vec![Color::Red],
            // Only round brackets configured; square and curly are not.
            pairs: vec![BracketPair::new('(', ')')],
        };
        let result = rb.colorize("[x]{y}");
        // No ANSI codes at all — `[`, `]`, `{`, `}` are all plain.
        assert!(!result.contains('\x1b'));
        assert_eq!(result, "[x]{y}");
    }

    // A close character that is a close in *some* pair but not the one currently open
    // must not consume the stack entry.
    #[test]
    fn unconfigured_close_does_not_consume_stack() {
        let rb = RainbowBrackets {
            colors: vec![Color::Red],
            pairs: vec![BracketPair::new('(', ')')],
        };
        // `[` is not a configured open, so it's plain; `)` correctly closes `(`.
        let result = rb.colorize("([x])");
        // The outer `(` and `)` should be colorized.
        assert_eq!(result.matches("\x1b[31m").count(), 2);
        // `[` and `]` are plain.
        assert!(result.contains("[x]"));
    }
}
