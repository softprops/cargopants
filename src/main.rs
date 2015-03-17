extern crate cargopants;

use cargopants::Client;

fn main() {
   let mut cargo = Client::new();   
   let mut k = cargo.krate("url");
   let v = k.version("0.2.25");
   println!("krate {:?}", v.get().unwrap());
}
