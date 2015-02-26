# cargo pants

> a comfortable outfit for storehousing crate orders and receipts

Cargo pants an an comfortably fitting interface for [crates.io](https://crates.io/). Some requests require authentication in the form of
an api token, to obtain one, visit [this page](https://crates.io/me) in your web browser.

## usage

```rust
extern crate cargopants;

use cargopants::Client;

fn main() {
   let mut cargo = Client::new().token("ap1k3y");
   println!("{:?}", cargo.reverse_dependencies("url").ok().expect("..."));
}
```

Doug Tangren (softprops) 2015
