extern crate warehouse;

use warehouse::Crates;

fn main() {
   let mut crates = Crates::new(
     "https://crates.io".to_string(), None);
   let get = crates.named("url");
   println!("result {:?}", get);
   let search = crates.search(
       "url"
    );
   println!("result {:?}", search.ok().expect("crates"));
}
