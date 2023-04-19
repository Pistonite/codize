/// Macro for creating a code line
///
/// # Examples
/// ```
/// use codize::codeln;
///
/// assert_eq!("hello", codeln!("hello").to_string());
/// assert_eq!("world foo", codeln!(f "world {}", "foo").to_string());
/// ```
#[macro_export]
macro_rules! codeln {
    (f$($arg:tt)*) => {
        $crate::Code::Line(format!($($arg)*))
    };
    ($arg:literal) => {
        $crate::Code::Line($arg.to_owned())
    };
    () => {
        $crate::Code::Line("".to_owned())
    };
}

#[cfg(test)]
mod ut {
    #[test]
    fn empty() {
        let code = codeln!();
        assert_eq!("", code.to_string());
    }
    #[test]
    fn literal() {
        let code = codeln!("hello world");
        assert_eq!("hello world", code.to_string());
    }

    #[test]
    fn format_one() {
        let code = codeln!(f "hello {}", 1);
        assert_eq!("hello 1", code.to_string());
    }

    #[test]
    fn format_three() {
        let test = "world";
        let code = codeln!(f "hello {} {} {} {test}", 1, 2, 3);
        assert_eq!("hello 1 2 3 world", code.to_string());
    }
}
