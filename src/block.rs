use crate::{Code, Concat, Format, FormatCode};

/// A block of code with a starting line, ending line, and an indented body
#[derive(derivative::Derivative)]
#[derivative(Debug, PartialEq)]
pub struct Block {
    /// If this block should be connected to the end of a previous block
    /// (for example, `else {`)
    pub connect: bool,
    /// The start of the block (for example, `if (x) {`)
    pub start: String,
    /// The end of the block (for example, `}`)
    pub end: String,
    /// The body of the block. Usually the body is the part that gets indented
    concat_body: Concat,
    /// When to inline
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    inline_condition: Option<fn(&Block) -> bool>,
}

impl Block {
    /// Create a new block with empty body
    pub fn empty<TStart, TEnd>(start: TStart, end: TEnd) -> Self
    where
        TStart: ToString,
        TEnd: ToString,
    {
        Self {
            connect: false,
            start: start.to_string(),
            concat_body: Concat::empty(),
            end: end.to_string(),
            inline_condition: None,
        }
    }

    /// Create a new code block
    pub fn new<
        TStart,
        TBody,
        TEnd
    >(start: TStart, body: TBody, end: TEnd) -> Self 
where
    TStart: ToString,
        TEnd: ToString,
        TBody: IntoIterator,
        TBody::Item: Into<Code>,
    {
        Self {
            connect: false,
            start: start.to_string(),
            concat_body: Concat::new(body),
            end: end.to_string(),
            inline_condition: None,
        }
    }

    /// Set this block to start on the same line as the end of the previous block
    pub fn connected(mut self) -> Self {
        self.connect = true;
        self
    }

    /// Set a condition for displaying the block as one line
    pub fn inline_when(mut self, condition: fn(&Block) -> bool) -> Self
    {
        self.inline_condition = Some(condition);
        self
    }

    /// Set the inline condition to be always true
    pub fn inlined(mut self) -> Self {
        self.inline_condition = Some(|_| true);
        self
    }

    /// Get the body of the block
    #[inline]
    pub fn body(&self) -> &[Code] {
        &self.concat_body
    }

    /// Should the block be displayed in one line
    pub fn should_inline(&self) -> bool {
        if let Some(condition) = self.inline_condition {
            condition(self)
        } else {
            self.should_inline_intrinsic()
        }
    }

    /// Should intrinsicly inline the block
    ///
    /// This is used for blocks that only contain one line of code
    pub fn should_inline_intrinsic(&self) -> bool {
        self.body().len() == 1 && self.body()[0].should_inline()
    }
}

impl From<Block> for Code {
    fn from(x: Block) -> Self {
        Code::Block(Box::new(x))
    }
}

impl ToString for Block {
    fn to_string(&self) -> String {
        self.format()
    }
}

impl FormatCode for Block {
    fn size_hint(&self) -> usize {
        // add the body, start, and end
        self.concat_body.size_hint() + 2
    }

    fn format_into_vec_with(&self, format: &Format, out: &mut Vec<String>, connect: bool, indent: &str) {
        let connect = self.connect || connect;
        crate::append_line(out, &self.start, connect, indent);
        let should_inline = self.should_inline();

        if should_inline {
            for code in self.body() {
                code.format_into_vec_with(format, out, true, indent);
            }
        } else {
            // indent the body
            let i = format.indent;
            let new_indent = if i < 0 {
                format!("\t{indent}")
            } else {
                let i = i as usize;
                format!("{:i$}{indent}", "")
            };
            for code in self.body() {
                code.format_into_vec_with(format, out, false, &new_indent);
            }
        }
        crate::append_line(out, &self.end, should_inline, indent);
    }
}

/// Macro for creating [`Block`]s
///
/// # Examples
///
/// ```
/// use codize::cblock;
///
/// let expected =
/// "fn main() {
///     foo();
/// }";
///
/// let code = cblock!("fn main() {", [
///    "foo();",
/// ], "}");
/// assert_eq!(expected, code.to_string());
///
/// ```
///
/// Anything that implements `Into<Code>` can be used in the body.
/// It can also be anything that implements `IntoIterator` and returns `Into<Code>`.
///
/// You can call [`Block::connected`] to connect the start of the block with the end of the last block,
/// such as an `else` block.
/// ```
/// use codize::cblock;
///
/// let expected =
/// "fn foo(y: bool) {
///     if (x()) {
///         bar();
///     } else if (y) {
///         baz();
///     }
/// }";
///
/// let func = "x";
/// let code = cblock!("fn foo(y: bool) {", [
///    cblock!(format!("if ({func}()) {{"), [
///       "bar();",
///    ], "}"),
///    cblock!("else if (y) {", [
///       "baz();"
///    ], "}").connected(),
/// ], "}");
/// assert_eq!(expected, code.to_string());
///
/// ```
#[macro_export]
macro_rules! cblock {
    ($start:expr, [] , $end:expr) => {
        $crate::Block::empty($start, $end)
    };
    ($start:expr, [ $( $body:expr ),* $(,)? ] , $end:expr) => {
        $crate::Block::new($start, [ $($crate::Code::from($body)),* ], $end)
    };
    ($start:expr, $body:expr, $end:expr) => {
        $crate::Block::new($start, $body, $end)
    };
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    #[test]
    fn empty() {
        let code = cblock!("", [], "");
        // start and end on separate lines
        assert_eq!("\n", code.to_string());
    }

    #[test]
    fn empty_body() {
        let code = cblock!("fn main() {", [], "}");
        assert_eq!("fn main() {\n}", code.to_string());
    }

    #[test]
    fn different_types() {
        let code = cblock!("fn main() {", [
            "foo",
            "bar".to_string(),
            cblock!("if (x) {", [
                "baz",
                "qux".to_string(),
            ], "}"),
        ], "}");
        let expected = indoc! {"
            fn main() {
                foo
                bar
                if (x) {
                    baz
                    qux
                }
            }"};
        assert_eq!(expected, code.to_string());
    }

    #[test]
    fn iteratable() {
        let body = vec![
            cblock!("if (x()) {", [
                "bar();"
            ], "}"),
            cblock!("else if (y) {", [
                "baz();"
            ], "}").connected(),
        ];
        let code = cblock!("fn foo(y: bool) {", body, "}");
        let expected = indoc! {"
            fn foo(y: bool) {
                if (x()) {
                    bar();
                } else if (y) {
                    baz();
                }
            }"};
        assert_eq!(expected, code.to_string());
    }

    fn is_one_thing(block: &crate::Block) -> bool {
        block.body().len() == 1
    }

    #[test]
    fn inline_condition() {
        let body = vec![
            cblock!("if (x()) {", [
                "bar();"
            ], "}").inline_when(is_one_thing),
            cblock!("else if (y) {", [
                "baz();",
                "baz();"
            ], "}").connected().inline_when(is_one_thing),
        ];
        let code = cblock!("fn foo(y: bool) {", body, "}");
        let expected = indoc! {"
            fn foo(y: bool) {
                if (x()) { bar(); } else if (y) {
                    baz();
                    baz();
                }
            }"};
        assert_eq!(expected, code.to_string());

    }
}

