#![feature(old_io)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate hyper;
extern crate mime;
extern crate url;

use hyper::{ Client, Url };
use hyper::header::{ Accept, Authorization, ContentType, UserAgent, qitem };
use hyper::method::Method;
use mime::{ Attr, Mime, Value };
use mime::TopLevel::Application;
use mime::SubLevel::Json;
use rustc_serialize::json;
use std::old_io::IoError;
use std::result;

pub type Result<T> = result::Result<T, IoError>;

pub struct Crates {
  host: String,
  token: Option<String>
}

#[derive(RustcDecodable)]
#[derive(Debug)]
pub struct Crate {
  pub name: String,
  pub description: Option<String>,
  pub max_version: String
}

#[derive(RustcDecodable)]
struct CrateList {
  crates: Vec<Crate>
}

impl Crates {
  pub fn new(host: String, token: Option<String>) -> Crates {
    Crates { host: host, token: token }
  }
 
  pub fn search(&mut self, query: &str) -> Result<Vec<Crate>> {
    let body = try!(self.get(format!("/crates?q={}", query)));
    Ok(json::decode::<CrateList>(&body).unwrap().crates)
  }

  pub fn named(&mut self, name: &str) -> Result<String> { // can't decode this automagically because it contains the key "crate"!
    self.get(format!("/crates/{}", name))
  }

  fn get(&mut self, path: String) -> Result<String> {
    self.req(path, None, Method::Get)
  }

  fn req(&mut self, path: String, body: Option<&[u8]>, method: Method) -> Result<String> {
     let uri = Url::parse(&format!("{}/api/v1{}", self.host, path)).ok().expect("invalid url");
     // todo: send Authorization header when token is defined
     let mut res = match Client::new()
        .request(method, uri)
        .header(UserAgent("ro/0.1.0".to_string()))
        .header(Accept(vec![qitem(Mime(Application, Json, vec![(Attr::Charset, Value::Utf8)]))]))
        .header(ContentType(Mime(Application, Json, vec![(Attr::Charset, Value::Utf8)])))
        .send() {
            Ok(r) => r,
            Err(err) => panic!("failed request: {:?}", err)
        };
      res.read_to_string()
  }  
}

#[test]
fn it_works() {
}
