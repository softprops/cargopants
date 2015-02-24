extern crate cargopants;

use cargopants::Client;

fn main() {
   let mut crates = Client::new();
   let get = crates.named("url");
   println!("result {:?}", get);
   let query = crates.find(
       "url"
    );
   println!("result {:?}", query.ok().expect("crates"));
}
