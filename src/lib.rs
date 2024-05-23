#![doc = include_str!("../README.md")]

mod block;
pub use block::Block;
mod concat;
pub use concat::Concat;
mod list;
pub use list::{List, Trailing};

/// Code structure
///
/// You should use the macros or `into` conversion instead of constructing this directly.
#[derive(Debug, PartialEq)]
pub enum Code {
    /// A line of code.
    Line(String),
    /// A block of code. See [`Block`]
    Block(Box<Block>),
    /// Concatenation of multiple code sections. See [`Concat`]
    Concat(Concat),
    /// A list of code segments with separator. See [`List`]
    List(List),
}

impl From<String> for Code {
    fn from(x: String) -> Self {
        Code::Line(x)
    }
}

impl From<&str> for Code {
    fn from(x: &str) -> Self {
        Code::Line(x.to_owned())
    }
}

/// Formatting options
#[derive(derivative::Derivative)]
#[derivative(Debug, PartialEq, Default)]
pub struct Format {
    /// The number of spaces to indent per level. `-1` to use tabs
    #[derivative(Default(value = "4"))]
    pub indent: i32,
}

impl Format {
    /// Set indent
    pub fn indent(indent: i32) -> Self {
        Self::default().set_indent(indent)
    }
    /// Set indent
    #[inline]
    pub fn set_indent(mut self, indent: i32) -> Self {
        self.indent = indent;
        self
    }
    /// Set indent to tabs
    pub fn indent_tab() -> Self {
        Self::indent(-1)
    }
    /// Set indent to tabs
    #[inline]
    pub fn set_indent_tab(self) -> Self {
        self.set_indent(-1)
    }
}

/// Enable different formatting options for [`Code`] structures
pub trait FormatCode {
    /// Emit self with the default format as a string
    fn format(&self) -> String {
        self.format_with(&Format::default())
    }

    /// Emit self with the format as a string
    fn format_with(&self, format: &Format) -> String {
        self.format_vec_with(format).join("\n")
    }
    /// Emit self with the format as a vector of lines
    fn format_vec_with(&self, format: &Format) -> Vec<String> {
        let size_hint = self.size_hint();
        let mut out = match size_hint {
            0 => Vec::new(),
            n => Vec::with_capacity(n),
        };
        self.format_into_vec_with(format, &mut out, false, "");
        // ensure no reallocation
        #[cfg(test)]
        if size_hint > 0 {
            assert_eq!(out.capacity(), size_hint);
        }
        out
    }
    /// Emit self with the format in the given output context
    fn format_into_vec_with(
        &self,
        format: &Format,
        out: &mut Vec<String>,
        connect: bool,
        indent: &str,
    );
    /// Upperbound for the line count of the code for pre-allocating. Return 0 to skip
    fn size_hint(&self) -> usize;
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl FormatCode for Code {
    fn format_into_vec_with(
        &self,
        format: &Format,
        out: &mut Vec<String>,
        connect: bool,
        indent: &str,
    ) {
        match self {
            Code::Line(line) => append_line(out, line, connect, indent),
            Code::Block(body) => body.format_into_vec_with(format, out, connect, indent),
            Code::Concat(body) => body.format_into_vec_with(format, out, connect, indent),
            Code::List(body) => body.format_into_vec_with(format, out, connect, indent),
        }
    }

    fn size_hint(&self) -> usize {
        match self {
            Code::Line(_) => 1,
            Code::Block(body) => body.size_hint(),
            Code::Concat(body) => body.size_hint(),
            Code::List(body) => body.size_hint(),
        }
    }
}

/// Helper function to append one line to the output within the given context
pub(crate) fn append_line(out: &mut Vec<String>, line: &str, connect: bool, indent: &str) {
    if connect {
        if let Some(last) = out.last_mut() {
            if !last.is_empty() && last != indent {
                last.push(' ');
            }
            last.push_str(line.as_ref());
            return;
        }
    }
    // when making a new line, make sure the previous line is not indented if it's only whitespaces
    if let Some(last) = out.last_mut() {
        if last.trim().is_empty() {
            "".clone_into(last);
        }
    }
    if indent.is_empty() {
        out.push(line.to_owned());
    } else {
        out.push(format!("{indent}{line}"));
    }
}

impl Code {
    /// Should the code be displayed in one line
    pub fn should_inline(&self) -> bool {
        match self {
            Code::Block(block) => block.should_inline(),
            Code::List(list) => list.should_inline(),
            _ => false,
        }
    }

    /// Get if this structure will generate any code or not (empty = no code)
    pub fn is_empty(&self) -> bool {
        match self {
            Code::Concat(concat) => concat.is_empty(),
            Code::List(list) => list.is_empty(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use super::*;

    fn test_case_1() -> Code {
        cblock!("{", [], "}").into()
    }

    fn test_case_2() -> Code {
        cblock!("trait A {", ["fn a();"], "}").into()
    }

    fn test_case_3() -> Code {
        cblock!(
            "fn main() {",
            [
                cblock!("if (foo) {", ["println!(\"Hello, world!\");"], "}"),
                cblock!("else {", [format!("bar({});", "giz")], "}").connected(),
            ],
            "}"
        )
        .into()
    }

    fn test_case_4(f1: fn(&Block) -> bool, f2: fn(&List) -> bool) -> Code {
        let body = vec![
            Code::from("let x = 1;"),
            cblock!(
                "let b = {",
                [clist!("," => ["1", "2", "3"]).inline_when(f2)],
                "};"
            )
            .inline_when(f1)
            .into(),
            cblock!(
                "let b = {",
                [clist!("," => ["1", "2", "3", "4"]).inline_when(f2)],
                "};"
            )
            .inline_when(f1)
            .into(),
        ];
        cblock!("while true {", body, "}").into()
    }

    #[test]
    fn test1() {
        let code = test_case_1();
        assert_eq!("{\n}", code.to_string());
    }

    #[test]
    fn test2() {
        let code = test_case_2();
        let expected = indoc! {"
            trait A {
               fn a();
            }"};
        assert_eq!(expected, code.format_with(&Format::indent(3)));
        let expected = indoc! {"
            trait A {
            \tfn a();
            }"};
        assert_eq!(expected, code.format_with(&Format::indent_tab()));
    }

    #[test]
    fn test3() {
        let code: Code = test_case_3();
        let expected = indoc! {"
            fn main() {
                if (foo) {
                    println!(\"Hello, world!\");
                } else {
                    bar(giz);
                }
            }"};
        assert_eq!(expected, code.to_string());
    }

    #[test]
    fn test4() {
        fn should_inline_list(list: &List) -> bool {
            list.body().len() == 3
        }
        let code = test_case_4(Block::should_inline_intrinsic, should_inline_list);
        let expected = indoc! {"
            while true {
                let x = 1;
                let b = { 1, 2, 3 };
                let b = {
                    1,
                    2,
                    3,
                    4,
                };
            }"};
        assert_eq!(expected, code.to_string());

        let code = test_case_4(Block::should_inline_intrinsic, |_| true);
        let expected = indoc! {"
            while true {
                let x = 1;
                let b = { 1, 2, 3 };
                let b = { 1, 2, 3, 4 };
            }"};
        assert_eq!(expected, code.to_string());

        let code = test_case_4(|_| false, |_| true);
        let expected = indoc! {"
            while true {
                let x = 1;
                let b = {
                    1, 2, 3
                };
                let b = {
                    1, 2, 3, 4
                };
            }"};
        assert_eq!(expected, code.to_string());
    }
}
