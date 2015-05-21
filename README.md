# cargo pants

[![Build Status](https://travis-ci.org/softprops/cargopants.svg?branch=master)](https://travis-ci.org/softprops/cargopants)

> a comfortable outfit for storehousing crate orders and receipts

Cargo pants is a comfortably fitting interface for [crates.io](https://crates.io/). Some requests require authentication in the form of
an API token, to obtain one, visit [this page](https://crates.io/me) in your web browser.

## docs

Find them [here](https://softprops.github.io/cargopants).

## usage

```rust
extern crate cargopants;

use cargopants::Client;

fn main() {
   let mut cargo = Client::new();
   println!("{:?}", cargo.reverse_dependencies("url").ok().expect("..."));
}
```

Doug Tangren (softprops) 2015
