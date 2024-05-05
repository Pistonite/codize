# codize

![Build Badge](https://img.shields.io/github/actions/workflow/status/Pistonite/codize/rust.yml)
![Version Badge](https://img.shields.io/crates/v/codize)
![License Badge](https://img.shields.io/github/license/Pistonite/codize)
![Issue Badge](https://img.shields.io/github/issues/Pistonite/codize)

Simple, language-agnostic library that pretty-prints code for your code-generation tool.

First, create a [`Code`] enum with the code structure with one of the ways listed below.
Then, the [`Format`] struct can be used to format the output. Or you can simply use `to_string`
for a quick formatting with the default parameters

## [`Code`] Examples
The [`Code`] enum stores all of the code structures. You can create it in one of the following
ways:
- Create a single line from a [`String`] or `&str` with `into()`
- A block of code with an indented body with the [`cblock!`] macro
- A list of code segments with a separator with the [`clist!`] macro
- A concatenation of multiple code segments, either converted from an iterator with `into()`,
or with the [`cconcat!`] macro which allows for mixing different types of code segments

Usually, the macros will automatically convert the input to [`Code`] by calling `Code::from`.

```rust
use codize::{cblock, clist, cconcat};

let code = cconcat![
    "",
    "/// This block is auto-generated",
    "",
    cblock!("fn main() {", [
        cblock!("println!(", [
            clist!("," => [r#""{}, {}!""#, r#""Hello""#, r#""world!""#])
        ], ");")
    ], "}"),
];

let expected = r#"
/// This block is auto-generated

fn main() {
    println!(
        "{}, {}!",
        "Hello",
        "world!",
    );
}"#;

assert_eq!(expected, code.to_string());

```

## [`Format`] Examples
You can use the [`FormatCode`] trait along with the [`Format`] struct to change global
formatting options
```rust
use codize::{cblock, Format, FormatCode};
let code = cblock!("fn main() {", ["println!(\"Hello, world!\");"], "}");

let indent_2 = 
r#"fn main() {
  println!("Hello, world!");
}"#;
assert_eq!(indent_2, code.format_with(&Format::indent(2)));

let indent_tab =
"fn main() {
\tprintln!(\"Hello, world!\");
}";
assert_eq!(indent_tab, code.format_with(&Format::indent_tab()));
```
