use crate::{Code, Concat, Format, FormatCode};

/// A list of code segments separated by a separator
#[derive(derivative::Derivative)]
#[derivative(Debug, PartialEq)]
pub struct List {
    /// The items in the list
    concat_body: Concat,
    /// The separator between the items
    pub separator: String,
    /// The trailing mode
    pub trailing: Trailing,
    /// When to inline
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    inline_condition: Option<fn(&List) -> bool>,
}

/// Trailing mode for a code list
#[derive(Debug, Clone, PartialEq)]
pub enum Trailing {
    /// Add trailing separator if the list is split into multiple lines
    IfMultiLine,
    /// Always add trailing separator
    Always,
    /// Never add trailing separator
    Never,
}

impl List {
    /// Create a new empty code list
    pub fn empty<TSep: ToString>(sep: TSep) -> Self {
        Self {
            separator: sep.to_string(),
            concat_body: Concat::empty(),
            trailing: Trailing::IfMultiLine,
            inline_condition: None,
        }
    }

    /// Create a new code list
    pub fn new<TSep, TBody>(sep: TSep, body: TBody) -> Self 
where
    TSep: ToString,
        TBody: IntoIterator,
        TBody::Item: Into<Code>,
    {
        Self {
            separator: sep.to_string(),
            concat_body: Concat::new(body),
            trailing: Trailing::IfMultiLine,
            inline_condition: None,
        }
    }

    /// Create a new code list with no trailing separator
    pub fn no_trail(mut self) -> Self {
        self.trailing = Trailing::Never;
        self
    }

    /// Create a new code list with trailing separator even if the list is in one line
    pub fn always_trail(mut self) -> Self {
        self.trailing = Trailing::Always;
        self
    }

    /// Set a condition for displaying the block as one line
    pub fn inline_when(mut self, condition: fn(&List) -> bool) -> Self
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

    /// Get if the list will generate any code or not (empty = no code)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.concat_body.is_empty()
    }

    /// Should the list be displayed in one line
    pub fn should_inline(&self) -> bool {
        if let Some(condition) = self.inline_condition {
            condition(self)
        } else {
            self.should_inline_intrinsic()
        }
    }

    /// Should intrinsicly inline the list
    ///
    /// This is used for lists that only contain one item
    pub fn should_inline_intrinsic(&self) -> bool {
        self.body().len() == 1 && self.body()[0].should_inline()
    }
}

impl From<List> for Code {
    #[inline]
    fn from(x: List) -> Self {
        Code::List(x)
    }
}

impl ToString for List {
    fn to_string(&self) -> String {
        self.format()
    }
}

impl FormatCode for List {
    fn size_hint(&self) -> usize {
        self.concat_body.size_hint()
    }

    fn format_into_vec_with(&self, format: &Format, out: &mut Vec<String>, connect: bool, indent: &str) {
        let should_inline = self.should_inline();

        // if first item is appended
        // used to check if separator should be added
        let mut first_appended = false;
        // should next item be connected to the previous one
        let mut previous_allow_connect = connect;

        let mut previous_size = out.len();
        let initial_size = previous_size;
        
        for code in self.body().iter().filter(|c| !c.is_empty()) {
            // append separator if needed
            if let Some(last) = out.last_mut() {
                if first_appended {
                    last.push_str(&self.separator);
                }
            }
            let connect = if first_appended {
            should_inline || (previous_allow_connect && {
                // allow connect if the item is first, not block, or is non-inline block
                    match code {
                    Code::Block(b) => !b.should_inline(),
                    _ => true
                }
            })
            } else {
                // for first, inline if connect
                connect
            };
            // emit the next item to out
            code.format_into_vec_with(format, out, connect, indent);
            // check if next item can be connected
            // only connect if the current is multi-line
            let new_size = out.len();
            previous_allow_connect = new_size > previous_size + 1;
            previous_size = new_size;
            first_appended = true;
        }
        
        let should_trail = match self.trailing {
            Trailing::IfMultiLine => previous_size > initial_size + 1,
            Trailing::Always => true,
            Trailing::Never => false,
        };
        if should_trail {
            if let Some(last) = out.last_mut() {
                last.push_str(&self.separator);
            }
        }
    }
}

/// Macro for creating [`List`]s
///
/// Note that spaces and newlines are automatically added between the items after the separator.
/// You don't need to specify them as part of the separator.
///
/// The default trailing separator behavior is only trail if the list is split into multiple lines.
/// You can use [`List::no_trail`] or [`List::always_trail`] to change the behavior.
///
/// # Examples
///
/// ```
/// use codize::{clist, cblock};
///
/// let expected = "call_something( a, b, c )";
/// let code = cblock!("call_something(", [
///     clist!("," => ["a", "b", "c"]).inlined()
/// ], ")");
/// assert_eq!(expected, code.to_string());
///
/// let expected = 
/// "call_something(
///     a,
///     b,
///     c,
/// )";
/// let code = cblock!("call_something(", [
///     clist!("," => ["a", "b", "c"])
/// ], ")");
/// assert_eq!(expected, code.to_string());
/// ```
#[macro_export]
macro_rules! clist {
    ($sep:expr => []) => {
        $crate::List::empty($sep)
    };
    ($sep:expr => [ $( $body:expr ),* $(,)? ]) => {
        $crate::List::new($sep, [ $($crate::Code::from($body)),* ])
    };
    ($sep:expr => $body:expr) => {
        $crate::List::new($sep, $body)
    };
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use crate::{cblock, Block, Code, List};

    #[test]
    fn empty() {
        let code = clist!("," => []);
        assert_eq!("", code.to_string());
    }

    #[test]
    fn one() {
        let code = clist!("," => ["hello"]);
        assert_eq!("hello", code.to_string());
    }

    #[test]
    fn one_trail() {
        let code = clist!("," => ["hello"]).always_trail();
        assert_eq!("hello,", code.to_string());
    }

    #[test]
    fn many() {
        let code = clist!("," => ["hello", "hello2"]);
        assert_eq!("hello,\nhello2,", code.to_string());
    }

    #[test]
    fn many_no_trail() {
        let code = clist!("," => ["hello", "hello2"]).no_trail();
        assert_eq!("hello,\nhello2", code.to_string());
    }

    #[test]
    fn with_blocks() {
        // the first block is inline, so next block cannot be connected
        let expected = indoc! {"
            { a },
            {
                hello,
                hello2
            }, {
                foo,
                bar,
            },
            { b },
            { c },"};

        let expected_inline = indoc! {"
            { a }, {
                hello,
                hello2
            }, {
                foo,
                bar,
            }, { b }, { c },"};

        fn should_inline_block(c: &Block) -> bool {
            if let Some(Code::Line(s)) = c.body().first() {
                s.len() == 1
            } else {
                false
            }
        }
        let code = clist!("," => [
            cblock!("{", ["a"], "}").inline_when(should_inline_block),
            cblock!("{", [clist!("," => ["hello", "hello2"]).no_trail()], "}"),
            cblock!("{", [clist!("," => ["foo", "bar"])], "}"),
            cblock!("{", ["b"], "}").inline_when(should_inline_block),
            cblock!("{", ["c"], "}").inline_when(should_inline_block),
        ]);
        assert_eq!(expected, code.to_string());
        assert_eq!(expected_inline, code.inline_when(|_| true).to_string());
    }

    #[test]
    fn multiple_levels() {
        let expected = indoc! {"
            a, b, c,
            d, e, f,
            x, y, z,"};
        fn always(_: &List) -> bool {
            true
        }
        let code = clist!("," => [
            clist!("," => ["a", "b", "c"]).inline_when(always),
            clist!("," => ["d", "e", "f"]).inline_when(always),
            clist!("," => ["x", "y", "z"]).inline_when(always),
        ]);
        assert!(!code.should_inline());
        assert_eq!(expected, code.to_string());
    }
}
