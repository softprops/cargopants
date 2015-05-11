#![deny(missing_docs)]
#![feature(test)]

//! Cargopants exposes a client interface for crates.io providing
//! open access to the rust community's crate inventory
//!
//! # examples
//!
//! ```
//! extern crate cargopants;
//!
//! let mut cargo = cargopants::Client::new();
//! let mut url = cargo.krate("url");
//!
//! println!("url@0.2.25 -> {:?} ", url.version("0.2.25").get().unwrap());
//! ```

extern crate hyper;
extern crate mime;
extern crate rustc_serialize;
extern crate test;

pub mod rep;

use std::ops::DerefMut;
use hyper::Url;
use hyper::client;
use hyper::header::{
  Accept, Authorization, ContentType,
  UserAgent, qitem
};
use hyper::method::Method;
use mime::{ Attr, Mime, Value };
use mime::TopLevel::Application;
use mime::SubLevel::Json;
use rustc_serialize::{ Decoder, Decodable, json };
use rep::{
  Crate, Dependency, Download,
  NewCrate, User, Version
};
use std::fs::{ self, File };
use std::io::prelude::*;
use std::io::{ Cursor, Error, Result };
use std::path::Path;
use std::result;

// internal representations of various 
// requests and responses

#[derive(RustcDecodable)]
struct VersionReq {
  version: Version
}

#[derive(RustcDecodable)]
struct Versions {
  versions: Vec<Version>
}

#[derive(RustcDecodable)]
struct Status {
  ok: bool
}

#[derive(RustcDecodable)]
struct Users {
  users: Vec<User>
}

#[derive(RustcDecodable)]
struct Following {
  following: bool
}

#[derive(RustcDecodable)]
struct Dependencies {
  dependencies: Vec<Dependency>    
}

#[derive(RustcDecodable)]
struct VersionDownloads {
  version_downloads: Vec<Download>
}

#[derive(RustcDecodable)]
struct ExtraDownloads {
  extra_downloads: Vec<Download>
}

#[derive(RustcDecodable)]
struct MetaDownloads {
  meta: ExtraDownloads
}

#[derive(RustcDecodable)]
struct Crates {
  crates: Vec<Crate>
}

// RustcDecodable be derived because the key used in json is `crate`,
// a reserved word
struct CrateReq {
  krate: Crate
}

impl Decodable for CrateReq {
  fn decode<D: Decoder>(d: &mut D) -> result::Result<CrateReq, D::Error> {
    d.read_struct("CrateReq", 1usize, |_d| {
      Ok(CrateReq {
        krate: try!(_d.read_struct_field("crate", 0usize, |_d| Decodable::decode(_d)))
      })
    })
  }
}

#[derive(RustcDecodable)]
struct Authors {
  meta: Meta
}

#[derive(RustcDecodable)]
struct Meta {
  names: Vec<String>
}

#[derive(RustcEncodable)]
struct OwnersReq<'a> {
  users: &'a [&'a str]
}

/// Entry point for accessing crates.io
pub struct Client {
  transport: Box<Transport>,
  token: Option<String>
}

#[doc(hidden)]
pub trait Transport {
  fn request(&mut self, method: Method, path: String, body: Option<Body>, token: Option<String>) -> Result<String>;
}

impl Transport for (hyper::Client, String) {
  fn request(&mut self, method: Method, path: String, body: Option<Body>, token: Option<String>) -> Result<String> {
    let uri = Url::parse(&format!("{}/api/v1{}", self.1, path)).ok().expect("invalid url");
    let content_type: Mime = Mime(Application, Json, vec![(Attr::Charset, Value::Utf8)]);
    let bound = self.0.request(method, uri)
      .header(UserAgent("cargopants/0.1.0".to_string()))
      .header(Accept(vec![qitem(content_type.clone())]))
      .header(ContentType(content_type));
    let authenticated = match token.clone() {
      Some(auth) => bound.header(Authorization(auth)),
               _ => bound
    };
    let embodied = match body {
      Some(Body { read: r, size: l }) => {
        let reader: &mut Read  = *r.deref_mut();
        authenticated.body(client::Body::SizedBody(reader, l))
      },
      _  => authenticated
    };
    let mut res = match embodied.send() {
      Ok(r)    => r,
      Err(err) => panic!("failed request: {:?}", err)
    };
    let mut body = String::new();
    res.read_to_string(&mut body).map(|_| body)
  }
}

#[doc(hidden)]
pub struct Body<'a> {
 read: &'a mut Box<&'a mut Read>,
 size: u64
}

impl<'a> Body<'a> {
  /// Create a new body instance
  pub fn new(read: &'a mut Box<&'a mut Read>, size: u64) -> Body<'a> {
    Body { read: read, size: size }
  }
}

/// Client interface for a given crate version
pub struct KrateVersion<'a, 'b, 'c> {
  client: &'a mut Client,
  name: &'b str,
  version: &'c str
}

impl<'a, 'b, 'c> KrateVersion<'a, 'b, 'c> {
  /// Provide a create version specific view of information
  pub fn new(client:&'a mut Client, name: &'b str, version: &'c str) -> KrateVersion<'a, 'b, 'c> {
    KrateVersion { client: client, name: name, version: version }
  }

  /// Fetch base set of information for a crate version
  pub fn get(self) -> Result<Version> {
    let body = try!(self.client.get(format!("/crates/{}/{}", self.name, self.version)));
    Ok(json::decode::<VersionReq>(&body).unwrap().version)
  }

  /// Fetch dependencies associated with a crate version
  pub fn dependencies(self) -> Result<Vec<Dependency>> {
    let body = try!(self.client.get(format!("/crates/{}/{}/dependencies", self.name, self.version)));
    Ok(json::decode::<Dependencies>(&body).unwrap().dependencies)
  }

  /// Fetch download information associated with a crate version
  pub fn downloads(self) -> Result<Vec<Download>> {
    let body = try!(self.client.get(format!("/crates/{}/{}/downloads", self.name, self.version)));
    Ok(json::decode::<VersionDownloads>(&body).unwrap().version_downloads)
  }

  /// Fetch authors associated with a crate version
  pub fn authors(self) -> Result<Vec<String>> {
    let body = try!(self.client.get(format!("/crates/{}/{}/authors", self.name, self.version)));
    Ok(json::decode::<Authors>(&body).unwrap().meta.names)
  }

  /// Yank a crate version from a crate server
  pub fn yank(self) -> Result<()> {
    let body = try!(self.client.delete(format!("/crates/{}/{}/yank", self.name, self.version), None));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  /// Unyank a crate version from a crate server
  pub fn unyank(self) -> Result<()> {
    let body = try!(self.client.put(format!("/crates/{}/{}/unyank", self.name, self.version), None));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }
}

/// Client interface for a given crate
pub struct Krate<'a, 'b> {
  client: &'a mut Client,
  name: &'b str
}

impl<'a, 'b> Krate<'a, 'b> {
  /// Provides a crate specific view of information
  pub fn new(client:&'a mut Client, name: &'b str) -> Krate<'a,'b> {
    Krate { client: client, name: name }
  }
  
  /// Request download information for a create
  pub fn downloads(self) -> Result<Vec<Download>> {
    let body = try!(self.client.get(format!("/crates/{}/downloads", self.name)));
    Ok(json::decode::<MetaDownloads>(&body).unwrap().meta.extra_downloads)
  }

  /// Follow a crate
  pub fn follow(self) -> Result<()> {
    let body = try!(self.client.put(format!("/crates/{}/follow", self.name), None));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  /// Unfollow a crate
  pub fn unfollow(self) -> Result<()> {
    let body = try!(self.client.delete(format!("/crates/{}/follow", self.name), None));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  /// Request indication of whether the current authentication credentials follows this crate
  pub fn following(self) -> Result<bool> {
    let body = try!(self.client.get(format!("/crates/{}/following", self.name)));
    Ok(json::decode::<Following>(&body).unwrap().following)
  }

  /// Get the base set of information associated for a crate
  pub fn get(self) -> Result<Crate> {
    let body = try!(self.client.get(format!("/crates/{}", self.name)));
    Ok(json::decode::<CrateReq>(&body).unwrap().krate)
  }

  /// Requests a vector of owners for a crate
  pub fn owners(self) -> Result<Vec<User>> {
    let body = try!(self.client.get(format!("/crates/{}/owners", self.name)));
    Ok(json::decode::<Users>(&body).unwrap().users)
  }

  /// Adds owner to for a crate
  pub fn add_owners(self, owners: &[&str]) -> Result<()> {
    let data = json::encode(&OwnersReq { users: owners }).unwrap();
    let mut bytes = data.as_bytes();
    let body = try!(self.client.put(format!("/crates/{}/owners", self.name),
                             Some(Body::new(&mut Box::new(&mut bytes), bytes.len() as u64))));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  /// Remove owners from a crate
  pub fn remove_owners(self, owners: &[&str]) -> Result<()> {
    let data = json::encode(&OwnersReq { users: owners }).unwrap();
    let mut bytes = data.as_bytes();
    let body = try!(self.client.delete(format!("/crates/{}/owners", self.name),
                                Some(Body::new(&mut Box::new(&mut bytes), bytes.len() as u64))));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  /// Fetches references to crates that depend on this crate
  pub fn reverse_dependencies(&mut self) -> Result<Vec<Dependency>> {
    let body = try!(self.client.get(format!("/crates/{}/reverse_dependencies", self.name)));
    Ok(json::decode::<Dependencies>(&body).unwrap().dependencies)
  }

  /// Provides access to crate version specific resources
  pub fn version<'c>(&'c mut self, version: &'c str) -> KrateVersion {
    KrateVersion::new(self.client, self.name, version)
  }

  /// Requests all versions associated with a given create
  pub fn versions(self) -> Result<Vec<Version>> {
    let body = try!(self.client.get(format!("/crates/{}/versions", self.name)));
    let versions: Vec<Version> = json::decode::<Versions>(&body).unwrap().versions;
    Ok(versions)
  }
}

impl Client {
  /// Create a new Client interface for crates.io
  pub fn new() -> Client {
    Client::host("https://crates.io")
  }

  /// Create a new Client interface for a given host
  pub fn host(addr: &str) -> Client {
    let transport = (hyper::Client::new(), addr.to_string());
    Client {
      transport: Box::new(transport),
      token: None
    }
  }

  /// Authenticate requests with an auth token
  pub fn token(self, auth: &str) -> Client {
    Client {
      transport: self.transport,
      token: Some(auth.to_string())
    }
  }

  /// Provides access to crate-specific resources
  pub fn krate<'a>(&'a mut self, name: &'a str) -> Krate {
    Krate::new(self, name)
  }

  // todo: sort (downloads|name), by letter/keyword/user_id/following
  /// Issues a request to find a crate by name
  pub fn find(&mut self, query: &str) -> Result<Vec<Crate>> {
    let body = try!(self.get(format!("/crates?q={}&sort={}", query, "name")));
    Ok(json::decode::<Crates>(&body).unwrap().crates)
  }

  // todo: publish -- https://github.com/rust-lang/crates.io/blob/dabd8778c1a515ea7572c59096da76e562afe2e2/src/lib.rs#L76
  /// Publishes a tar'd crate file to crate server
  pub fn publish(&mut self, krate: &NewCrate, tarball: &Path) -> Result<()> {
    let json = json::encode(krate).unwrap();
    let stat = try!(fs::metadata(tarball));
    let header = {
      let mut w = Vec::new();
      w.extend([
        (json.len() >>  0) as u8,
        (json.len() >>  8) as u8,
        (json.len() >> 16) as u8,
        (json.len() >> 24) as u8,
      ].iter().cloned());
      w.extend(json.as_bytes().iter().cloned());
      w.extend([
        (stat.len() >>  0) as u8,
        (stat.len() >>  8) as u8,
        (stat.len() >> 16) as u8,
        (stat.len() >> 24) as u8,
      ].iter().cloned());
      w
    };
    let size = stat.len() as usize + header.len();
    let tarball = try!(File::open(tarball));
    let mut body = Cursor::new(header).chain(tarball);
    let _ = try!(self.put("/crates/new".to_string(), Some(Body::new(&mut Box::new(&mut body), size as u64))));
    Ok(())
  }

  // todo: version download -- https://github.com/rust-lang/crates.io/blob/dabd8778c1a515ea7572c59096da76e562afe2e2/src/lib.rs#L78

  fn get(&mut self, path: String) -> Result<String> {
    self.req(Method::Get, path, None)
  }

  fn delete(&mut self, path: String, body: Option<Body>) -> Result<String> {
    self.req(Method::Delete, path, body)
  }

  fn put(&mut self, path: String, body: Option<Body>) -> Result<String> {
    self.req(Method::Put, path, body)
  }

  fn req(&mut self, method: Method, path: String, body: Option<Body>) -> Result<String> {
    self.transport.request(method, path, body, self.token.clone())
  }  
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_parses_crate_req() {
    
  }
}
