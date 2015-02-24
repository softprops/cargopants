#![feature(old_io)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate hyper;
extern crate mime;
extern crate url;

use hyper::Url;
use hyper::header::{ Accept, Authorization, ContentType, UserAgent, qitem };
use hyper::method::Method;
use mime::{ Attr, Mime, Value };
use mime::TopLevel::Application;
use mime::SubLevel::Json;
use rustc_serialize::json;
use std::old_io::IoError;
use std::result;

pub type Result<T> = result::Result<T, IoError>;

pub struct Client {
  host: String,
  token: Option<String>
}

#[derive(RustcDecodable)]
struct Status {
  ok: bool
}

#[derive(RustcDecodable)]
pub struct User {
  pub id: u32,
  pub login: String,
  pub avatar: String,
  pub email: Option<String>,
  pub name: Option<String>,
}

#[derive(RustcDecodable)]
struct Users {
  users: Vec<User>
}

#[derive(RustcDecodable)]
#[derive(Debug)]
pub struct Crate {
  pub name: String,
  pub description: Option<String>,
  pub max_version: String
}

#[derive(RustcEncodable)]
pub struct OwnersReq<'a> {
  users: &'a [&'a str]
}

#[derive(RustcDecodable)]
struct Crates {
  crates: Vec<Crate>
}

impl Client {
  pub fn new() -> Client {
    Client::host("https://crates.io")
  }

  pub fn host(addr: &str) -> Client {
    Client { host: addr.to_string(), token: None }
  }

  pub fn token(self, auth: &str) -> Client {
    Client {
      host: self.host,
      token: Some(auth.to_string())
    }
  }
 
  pub fn find(&mut self, query: &str) -> Result<Vec<Crate>> {
    let body = try!(self.get(format!("/crates?q={}", query)));
    Ok(json::decode::<Crates>(&body).unwrap().crates)
  }

  pub fn named(&mut self, name: &str) -> Result<String> { // can't decode this automagically because it contains the key "crate"!
    self.get(format!("/crates/{}", name))
  }

  pub fn add_owners(&mut self, krate: &str, owners: &[&str]) -> Result<()> {
    let body = json::encode(&OwnersReq { users: owners }).unwrap();
    try!(self.put(format!("/crates/{}/owners", krate),
                   body.as_bytes()));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  pub fn remove_owners(&mut self, krate: &str, owners: &[&str]) -> Result<()> {
    let body = json::encode(&OwnersReq { users: owners }).unwrap();
    try!(self.delete(format!("/crates/{}/owners", krate),
                     Some(body.as_bytes())));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  pub fn owners(&mut self, krate: &str) -> Result<Vec<User>> {
    let body = try!(self.get(format!("/crates/{}/owners", krate)));
    Ok(json::decode::<Users>(&body).unwrap().users)
  }

  fn get(&mut self, path: String) -> Result<String> {
    self.req(path, None, Method::Get)
  }

  fn delete(&mut self, path: String, body: Option<&[u8]>) -> Result<String> {
    self.req(path, body, Method::Delete)
  }

  fn put(&mut self, path: String, b: &[u8]) -> Result<String> {
    self.req(path, Some(b), Method::Put)
  }

  fn req(&mut self, path: String, body: Option<&[u8]>, method: Method) -> Result<String> {
     let uri = Url::parse(&format!("{}/api/v1{}", self.host, path)).ok().expect("invalid url");
     let mut cli = hyper::Client::new();
     let bound = cli.request(method, uri)
        .header(UserAgent("cargopants/0.1.0".to_string()))
        .header(Accept(vec![qitem(Mime(Application, Json, vec![(Attr::Charset, Value::Utf8)]))]))
        .header(ContentType(Mime(Application, Json, vec![(Attr::Charset, Value::Utf8)])));
      let authenticated = match self.token.clone() {
        Some(auth) => bound.header(Authorization(auth)),
                 _ => bound
      };
      let embodied = match body {
        Some(data) => authenticated.body(data),
                 _ => authenticated
      };
      let mut res = match embodied.send() {
        Ok(r) => r,
        Err(err) => panic!("failed request: {:?}", err)
      };
      res.read_to_string()
  }  
}

#[test]
fn it_works() {
}
