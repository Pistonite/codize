/// Macro for concatenating multiple code sections
///
/// # Examples
/// ```
/// use codize::{codeln, block, block_concat};
///
/// let expected = r"
/// fn main() {
///     foo();
/// }
///
/// fn foo() {
///     bar();
/// }";
///
/// let code = block_concat![
///     block!("\nfn main() {", [
///        codeln!("foo();"),
///     ], "}"),
///     codeln!(),
///     block!("fn foo() {", [
///         codeln!("bar();"),
///     ], "}")
/// ];
/// assert_eq!(expected, code.to_string());
#[macro_export]
macro_rules! block_concat {
    ($($args:tt)*) => {
        $crate::Code::Concat(vec![ $($args)* ])
    };
}

#[cfg(test)]
mod test {
    use crate::{block, codeln, Code};

    #[test]
    fn empty() {
        let code = block_concat!();
        assert_eq!(code, Code::Concat(vec![]));
    }

    #[test]
    fn one() {
        let code = block_concat!(codeln!("Hello, World!"));
        assert_eq!(code, Code::Concat(vec![codeln!("Hello, World!")]));
    }

    #[test]
    fn mixed() {
        let code = block_concat![
            codeln!("Hello, World!"),
            block!("if (x) {", [codeln!("y();")], "}"),
            codeln!(""),
            block!("if (x2) {", [codeln!("y2();")], "}"),
        ];
        assert_eq!(
            code,
            Code::Concat(vec![
                codeln!("Hello, World!"),
                block!("if (x) {", [codeln!("y();")], "}"),
                codeln!(),
                block!("if (x2) {", [codeln!("y2();")], "}"),
            ])
        );
    }
}
