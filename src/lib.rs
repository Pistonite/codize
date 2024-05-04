//! # codize
//! Simply library that helps with turning code into strings.
//!
//! See [`codeln!`], [`block!`] and [`block_concat!`] macros for examples.

mod block;
mod codeln;
mod concat;

/// Code block
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// If this block should be connected to the end of the previous block
    /// (for example, `else {`)
    pub connect: bool,
    /// The start of the block (for example, `if (x) {`)
    pub start: String,
    /// The body of the block. Usually the body is the part that gets indented
    pub body: Vec<Code>,
    /// The end of the block (for example, `}`)
    pub end: String,
}

impl Block {
    fn size_hint(&self) -> usize {
        let body_size: usize = self.body.iter().map(|code| code.size_hint()).sum();
        body_size + 2
    }
}

impl From<Block> for Code {
    fn from(block: Block) -> Self {
        Code::Block(Box::new(block))
    }
}

/// Code enum
#[derive(Debug, Clone, PartialEq)]
pub enum Code {
    /// A line of code.
    ///
    /// The content shouldn't contain the newline `\n` character.
    /// It will be automatically inserted when formatting with [`Codize`]
    Line(String),
    /// A block of code. See [`Block`]
    Block(Box<Block>),
    /// Concatenation of multiple code sections
    Concat(Vec<Code>),
}

/// Codize formatter
///
/// This turns a [`Code`] into a string, joined with newlines.
///
/// There are formatting options you can set through its methods or by directly constructing it.
pub struct Codize {
    /// The number of spaces to indent per level.
    pub indent: usize,
    /// A function that determines whether a block should be inlined.
    ///
    /// When the block is inlined, the body is put on the same line as the start and the end,
    /// and the body will be separated by a space instead of a newline.
    /// For example, you can set the block to inline if it only contains one line of code.
    ///
    /// Note that this function is called on blocks of all levels.
    /// You can use the block starting and ending strings to do basic filtering
    pub inline_condition: Box<dyn Fn(&Block) -> bool>,
    /// If a trailing newline should be added when there is none
    pub trailing_newline: bool,
}

impl Default for Codize {
    /// Create the default formatting
    ///
    /// The default formatting is:
    /// - Indentation of 4 spaces
    /// - Blocks are never inlined
    /// - No trailing newline
    fn default() -> Self {
        Self {
            indent: 4,
            inline_condition: Box::new(|_| false),
            trailing_newline: false,
        }
    }
}

impl Codize {
    /// Set indent
    #[inline]
    pub fn indent(indent: usize) -> Self {
        Self::default().set_indent(indent)
    }
    /// Set indent
    #[inline]
    pub fn set_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }
    /// Set the blocks to always inline
    #[inline]
    pub fn always_inline() -> Self {
        Self::default().set_always_inline()
    }
    /// Set the blocks to always inline
    #[inline]
    pub fn set_always_inline(mut self) -> Self {
        self.inline_condition = Box::new(|_| true);
        self
    }
    /// Set the inline condition
    #[inline]
    pub fn inline_when<F>(condition: F) -> Self
    where
        F: Fn(&Block) -> bool + 'static,
    {
        Self::default().set_inline_when(condition)
    }
    /// Set the inline condition
    #[inline]
    pub fn set_inline_when<F>(mut self, condition: F) -> Self
    where
        F: Fn(&Block) -> bool + 'static,
    {
        self.inline_condition = Box::new(condition);
        self
    }
    /// Set if there should be trailing newline
    #[inline]
    pub fn trailing_newline(trailing_newline: bool) -> Self {
        Self::default().set_trailing_newline(trailing_newline)
    }
    /// Set if there should be trailing newline
    #[inline]
    pub fn set_trailing_newline(mut self, trailing_newline: bool) -> Self {
        self.trailing_newline = trailing_newline;
        self
    }
}

impl std::fmt::Display for Code {
    /// Format the code with the default formatting
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_with(&Codize::default()))
    }
}

impl Code {
    pub fn block(block: Block) -> Self {
        block.into()
    }

    /// Should the code be put on the same line as the previous code
    #[inline]
    fn should_connect(&self) -> bool {
        match self {
            Code::Block(block) => block.connect,
            _ => false,
        }
    }

    /// Get the upperbound for the line count of the code
    fn size_hint(&self) -> usize {
        match self {
            Code::Line(line) => line.len(),
            Code::Block(block) => block.size_hint(),
            Code::Concat(codes) => codes.iter().map(|code| code.size_hint()).sum(),
        }
    }

    /// Convert the code to a [`String`] with the given formatting
    #[inline]
    pub fn to_string_with(&self, codize: &Codize) -> String {
        let mut str = self.to_vec_with(codize).join("\n");
        if codize.trailing_newline && !str.ends_with('\n') {
            str.push('\n');
        }
        str
    }
    /// Convert the code to a vector of [`String`]s with the given formatting
    ///
    /// Note that the vectors won't contain the new line characters, and the `trailing_newline` option is ignored.
    pub fn to_vec_with(&self, codize: &Codize) -> Vec<String> {
        let mut out = Vec::with_capacity(self.size_hint());
        self.append_to_vec_with(codize, &mut out);
        out
    }

    /// Append the code to a vector of [`String`]s with the given formatting
    ///
    /// Note that the vectors won't contain the new line characters, and the `trailing_newline` option is ignored.
    pub fn append_to_vec_with(&self, codize: &Codize, out: &mut Vec<String>) {
        match self {
            Code::Line(line) => out.push(line.to_owned()),
            Code::Concat(codes) => {
                for code in codes {
                    code.append_to_vec_with(codize, out);
                }
            }
            Code::Block(block) => {
                let should_inline = (codize.inline_condition)(block);
                let start = &block.start;
                let body = &block.body;
                let end = &block.end;
                out.push(start.clone());

                for code in body {
                    let sub_lines = code.to_vec_with(codize);
                    let should_connect = should_inline || code.should_connect();

                    let skip = if should_connect {
                        let last = out.last_mut().unwrap();
                        last.push(' ');
                        last.push_str(sub_lines.first().unwrap());
                        1
                    } else {
                        0
                    };

                    for line in sub_lines.into_iter().skip(skip) {
                        out.push(format!("{:>indent$}{}", "", line, indent = codize.indent));
                    }
                }
                if should_inline {
                    let last = out.last_mut().unwrap();
                    last.push(' ');
                    last.push_str(end);
                } else {
                    out.push(end.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod ut {
    use super::*;

    fn test_case_1() -> Code {
        Block {
            connect: false,
            start: "{".to_owned(),
            body: vec![],
            end: "}".to_owned(),
        }
        .into()
    }

    fn test_case_2() -> Code {
        Block {
            connect: false,
            start: "trait A {".to_owned(),
            body: vec![codeln!("fn a();")],
            end: "}".to_owned(),
        }
        .into()
    }

    fn test_case_3() -> Code {
        block!(
            "fn main() {",
            [
                block!("if (foo) {", [codeln!("println!(\"Hello, world!\");")], "}"),
                block!(> "else {", [
                codeln!(f "bar({});", "giz")
            ], "}"),
            ],
            "}"
        )
    }

    fn test_case_4() -> Code {
        let body = vec![
            codeln!("let x = 1;"),
            block!(
                "let b = {",
                [codeln!("1,"), codeln!("2,"), codeln!("3,")],
                "};"
            ),
            block!(
                "let b = {",
                [codeln!("1,"), codeln!("2,"), codeln!("3,"), codeln!("4,"),],
                "};"
            ),
        ];
        dynblock!("while true {", body, "}")
    }

    #[test]
    fn test1() {
        let code: Code = test_case_1();
        assert_eq!("{\n}", code.to_string());
        assert_eq!(
            "{\n}\n",
            code.to_string_with(&Codize::trailing_newline(true))
        );
    }

    #[test]
    fn test2() {
        let code: Code = test_case_2();
        assert_eq!(
            "trait A {\n   fn a();\n}",
            code.to_string_with(&Codize::indent(3))
        );
        assert_eq!(
            "trait A { fn a(); }\n",
            code.to_string_with(&Codize::trailing_newline(true).set_always_inline())
        );
    }

    #[test]
    fn test3() {
        let code: Code = test_case_3();
        assert_eq!("fn main() {\n   if (foo) {\n      println!(\"Hello, world!\");\n   } else {\n      bar(giz);\n   }\n}", code.to_string_with(&Codize::indent(3)));
    }

    #[test]
    fn test4() {
        let code: Code = test_case_4();
        assert_eq!("while true {\n  let x = 1;\n  let b = {\n    1,\n    2,\n    3,\n  };\n  let b = {\n    1,\n    2,\n    3,\n    4,\n  };\n}", code.to_string_with(&Codize::indent(2)));
        let cond_inlined = code.to_string_with(
            &Codize::indent(2)
                .set_inline_when(|block| block.start.starts_with("let") && block.body.len() == 3),
        );
        assert_eq!("while true {\n  let x = 1;\n  let b = { 1, 2, 3, };\n  let b = {\n    1,\n    2,\n    3,\n    4,\n  };\n}", cond_inlined);
    }
}
