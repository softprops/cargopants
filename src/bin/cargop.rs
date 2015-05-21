//#![deny(warnings)]

extern crate cargopants;

fn main() {
   let mut cargo = cargopants::Client::new();   
   let url = cargo.krate("url");
   println!("latest version {:?}", url.get().unwrap());
   println!("krate {:?}", url.version("0.2.25").get().unwrap());
}
