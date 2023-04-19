/// Macro for creating a code block
///
/// # Examples
///
/// ```
/// use codize::{codeln, block};
///
/// let expected = r"
/// fn main() {
///     foo();
/// }";
///
/// let code = block!("\nfn main() {", [
///    codeln!("foo();"),
/// ], "}");
/// assert_eq!(expected, code.to_string());
///
/// ```
/// ```
/// use codize::{codeln, block, Codize};
///
/// let expected = r"
/// fn foo(y: bool) {
///   if (x()) {
///     bar();
///   } else if (y) {
///     baz();
///   }
/// }
/// ";
///
/// let func = "x";
/// let code = block!("\nfn foo(y: bool) {", [
///    block!(format!("if ({func}()) {{"), [
///       codeln!("bar();"),
///    ], "}"),
///    block!(> "else if (y) {", [
///       codeln!("baz();")
///    ], "}"),
/// ], "}");
/// assert_eq!(expected, code.to_string_with(&Codize::indent(2).set_trailing_newline(true)));
///
/// ```

#[macro_export]
macro_rules! block {
    ($start:literal, [ $( $body:expr ),* $(,)? ] , $end:literal) => {
        $crate::Code::Block($crate::Block {
            connect: false,
            start: $start.to_owned(),
            body: vec![ $( $body ),* ],
            end: $end.to_owned(),
        })
    };
    (> $start:literal, [ $( $body:expr ),* $(,)? ] , $end:literal) => {
        $crate::Code::Block($crate::Block {
            connect: true,
            start: $start.to_owned(),
            body: vec![ $( $body ),* ],
            end: $end.to_owned(),
        })
    };
    ($start:expr, [ $( $body:expr ),* $(,)? ] , $end:literal) => {
        $crate::Code::Block($crate::Block {
            connect: false,
            start: $start,
            body: vec![ $( $body ),* ],
            end: $end.to_owned(),
        })
    };
    (> $start:expr, [ $( $body:expr ),* $(,)? ] , $end:literal) => {
        $crate::Code::Block($crate::Block {
            connect: true,
            start: $start,
            body: vec![ $( $body ),* ],
            end: $end.to_owned(),
        })
    };
    ($start:literal, [ $( $body:expr ),* $(,)? ] , $end:expr) => {
        $crate::Code::Block($crate::Block {
            connect: false,
            start: $start.to_owned(),
            body: vec![ $( $body ),* ],
            end: $end,
        })
    };
    (> $start:literal, [ $( $body:expr ),* $(,)? ] , $end:expr) => {
        $crate::Code::Block($crate::Block {
            connect: true,
            start: $start.to_owned(),
            body: vec![ $( $body ),* ],
            end: $end,
        })
    };
    ($start:expr, [ $( $body:expr ),* $(,)? ] , $end:expr) => {
        $crate::Code::Block($crate::Block {
            connect: false,
            start: $start,
            body: vec![ $( $body ),* ],
            end: $end,
        })
    };
    (> $start:expr, [ $( $body:expr ),* $(,)? ] , $end:expr) => {
        $crate::Code::Block($crate::Block {
            connect: true,
            start: $start,
            body: vec![ $( $body ),* ],
            end: $end,
        })
    };
}
