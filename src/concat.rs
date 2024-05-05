use std::ops::{Deref, DerefMut};

use crate::{Code, Format, FormatCode};

/// A concatenation of multiple code sections
#[derive(Debug, PartialEq)]
pub struct Concat {
    body: Vec<Code>,
}

impl Concat {
    /// Create a new empty concatenation of code sections
    pub fn empty() -> Self {
        Self { body: vec![] }
    }

    /// Create a new concatenation of code sections
    pub fn new<TBody>(body: TBody) -> Self
    where
        TBody: IntoIterator,
        TBody::Item: Into<Code>,
    {
        Self {
            body: body.into_iter().map(|code| code.into()).collect(),
        }
    }

    /// Get if the concat will generate any code or not (empty = no code)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }
}

impl Deref for Concat {
    type Target = Vec<Code>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl DerefMut for Concat {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}

impl From<Concat> for Code {
    #[inline]
    fn from(x: Concat) -> Self {
        Code::Concat(x)
    }
}

impl<T> From<T> for Concat
where
    T: IntoIterator,
    T::Item: Into<Code>,
{
    fn from(body: T) -> Self {
        Concat::new(body)
    }
}

impl std::fmt::Display for Concat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl FormatCode for Concat {
    fn size_hint(&self) -> usize {
        self.body.iter().map(|code| code.size_hint()).sum()
    }

    fn format_into_vec_with(
        &self,
        format: &Format,
        out: &mut Vec<String>,
        connect: bool,
        indent: &str,
    ) {
        let mut iter = self.body.iter();
        if let Some(first) = iter.next() {
            first.format_into_vec_with(format, out, connect, indent);
        }
        for code in iter {
            code.format_into_vec_with(format, out, false, indent);
        }
    }
}

/// Macro for creating [`Concat`]s
///
/// ### Note
/// When passing in 1 argument, it needs to be an iterator of sections (such as a `Vec<Code>`).
/// This variant is the same as calling `into()` or `Concat::from()`.
///
/// For 2 or more arguments, they are concatenated into a new [`Concat`] instance.
///
/// # Examples
/// ```
/// use codize::{cblock, cconcat};
///
/// let expected = r"fn main() {
///     foo();
/// }
///
/// fn foo() {
///     bar();
/// }";
///
/// let code = cconcat![
///     cblock!("fn main() {", [
///        "foo();",
///     ], "}"),
///     "",
///     cblock!("fn foo() {", [
///         "bar();",
///     ], "}")
/// ];
/// assert_eq!(expected, code.to_string());
#[macro_export]
macro_rules! cconcat {
    () => {
        $crate::Concat::empty()
    };
    ($body:expr)=> {
        $crate::Concat::new($body)
    };
    ($( $body:expr ),* $(,)?) => {
        $crate::Concat::new([ $($crate::Code::from($body)),* ])
    };
}

#[cfg(test)]
mod test {
    use crate::{cblock, Code, Concat};

    #[test]
    fn empty() {
        let code = cconcat![];
        assert_eq!(code, Concat::empty());
    }

    #[test]
    fn one() {
        let code = cconcat![["Hello, World!"]];
        assert_eq!(code, Concat::new(["Hello, World!"]));
    }

    #[test]
    fn mixed() {
        let code = cconcat![
            "Hello, World!",
            cblock!("if (x) {", ["y();"], "}"),
            "",
            cblock!("if (x2) {", ["y2();"], "}"),
        ];
        assert_eq!(
            code,
            Concat::new([
                Code::from("Hello, World!"),
                cblock!("if (x) {", ["y();"], "}").into(),
                "".into(),
                cblock!("if (x2) {", ["y2();"], "}").into(),
            ])
        );
    }
}
