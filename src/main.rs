extern crate cargopants;

use cargopants::Client;

fn main() {
   let mut crates = Client::new(
     "https://crates.io".to_string(), None);
   let get = crates.named("url");
   println!("result {:?}", get);
   let search = crates.search(
       "url"
    );
   println!("result {:?}", search.ok().expect("crates"));
}
