# codize

A simple library that helps with printing code as strings

```rust
use codize::{codeln, block};

let expected = r"
fn main() {
    foo();
}";

let code = block!("\nfn main() {", [
   codeln!("foo();"),
], "}");
assert_eq!(expected, code.to_string());
```