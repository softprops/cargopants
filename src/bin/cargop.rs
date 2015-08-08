//#![deny(warnings)]

extern crate cargopants;

fn main() {
   let mut cargo = cargopants::Client::new();
   println!("latest version {:?}", cargo.krate("url").get().unwrap());
   println!("krate {:?}", cargo.krate("url").version("0.2.25").get().unwrap());
}
