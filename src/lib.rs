#![feature(core, test)]

//! # cargopants
//! Cargopants exposes an client interface for crates.io

extern crate core;
extern crate hyper;
extern crate mime;
extern crate rustc_serialize;
extern crate test;
extern crate url;

use core::ops::DerefMut;
use hyper::Url;
use hyper::client;
use hyper::header::{ Accept, Authorization, ContentType, UserAgent, qitem };
use hyper::method::Method;
use mime::{ Attr, Mime, Value };
use mime::TopLevel::Application;
use mime::SubLevel::Json;
use rustc_serialize::{ Decoder, Decodable, json };
use std::collections::HashMap;
use std::fs::{ self, File };
use std::io::prelude::*;
use std::io::{ Cursor, Error };
use std::path::Path;
use std::result;

pub type Result<T> = result::Result<T, Error>;

/// Entry point for accessing crates.io
pub struct Client {
  transport: Box<Transport>,
  token: Option<String>
}

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

#[derive(RustcDecodable)]
struct Status {
  ok: bool
}

#[derive(RustcDecodable)]
struct Following {
  following: bool
}

/// Representation of a crates.io User
#[derive(RustcDecodable)]
#[derive(Debug)]
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

/// Representation of a crates.io Crate
#[derive(RustcDecodable)]
#[derive(Debug)]
pub struct Crate {
  pub name: String,
  pub description: Option<String>,
  pub max_version: String
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

/// Representation of the downloads of a version on a given date
#[derive(RustcDecodable)]
#[derive(Debug)]
pub struct Download {
  pub date: String,
  pub downloads: u32
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

/// Representation of a crate dependency
#[derive(RustcDecodable)]
#[derive(Debug)]
pub struct Dependency {
  pub crate_id: String,
  pub default_features: bool,
  pub features: String,
  pub kind: String,
  pub optional: bool,
  pub req: String  
}

#[derive(RustcDecodable)]
struct Dependencies {
  dependencies: Vec<Dependency>    
}

/// Representation of a crate version
#[derive(RustcDecodable)]
#[derive(Debug)]
pub struct Version {
  // crate: String,
  pub created_at: String,
  pub dl_path: String,
  pub downloads: u32,
  pub num: String,
  pub updated_at: String,
  pub yanked: bool
}

#[derive(RustcDecodable)]
struct VersionReq {
  version: Version
}

#[derive(RustcDecodable)]
struct Versions {
  versions: Vec<Version>
}

#[derive(RustcDecodable)]
struct Meta {
  names: Vec<String>
}

#[derive(RustcDecodable)]
struct Authors {
  meta: Meta
}

/// Interface for creating a new crate
#[derive(RustcEncodable)]
pub struct NewCrate {
  pub name: String,
  pub vers: String,
  pub deps: Vec<NewCrateDependency>,
  pub features: HashMap<String, Vec<String>>,
  pub authors: Vec<String>,
  pub description: Option<String>,
  pub documentation: Option<String>,
  pub homepage: Option<String>,
  pub readme: Option<String>,
  pub keywords: Vec<String>,
  pub license: Option<String>,
  pub license_file: Option<String>,
  pub repository: Option<String>,
}

/// Representation of a new crate dependency
#[derive(RustcEncodable)]
pub struct NewCrateDependency {
  pub optional: bool,
  pub default_features: bool,
  pub name: String,
  pub features: Vec<String>,
  pub version_req: String,
  pub target: Option<String>,
  pub kind: String,
}

#[derive(RustcEncodable)]
struct OwnersReq<'a> {
  users: &'a [&'a str]
}

#[derive(RustcDecodable)]
struct Crates {
  crates: Vec<Crate>
}

pub struct Body<'a> {
 read: &'a mut Box<&'a mut Read>,
 size: u64
}

impl<'a> Body<'a> {
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
  pub fn new(client:&'a mut Client, name: &'b str, version: &'c str) -> KrateVersion<'a, 'b, 'c> {
    KrateVersion { client: client, name: name, version: version }
  }

  pub fn get(self) -> Result<Version> {
    let body = try!(self.client.get(format!("/crates/{}/{}", self.name, self.version)));
    Ok(json::decode::<VersionReq>(&body).unwrap().version)
  }

  pub fn dependencies(self) -> Result<Vec<Dependency>> {
    let body = try!(self.client.get(format!("/crates/{}/{}/dependencies", self.name, self.version)));
    Ok(json::decode::<Dependencies>(&body).unwrap().dependencies)
  }

  pub fn downloads(self) -> Result<Vec<Download>> {
    let body = try!(self.client.get(format!("/crates/{}/{}/downloads", self.name, self.version)));
    Ok(json::decode::<VersionDownloads>(&body).unwrap().version_downloads)
  }

  pub fn authors(self) -> Result<Vec<String>> {
    let body = try!(self.client.get(format!("/crates/{}/{}/authors", self.name, self.version)));
    Ok(json::decode::<Authors>(&body).unwrap().meta.names)
  }

  pub fn yank(self) -> Result<()> {
    let body = try!(self.client.delete(format!("/crates/{}/{}/yank", self.name, self.version), None));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

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
  pub fn new(client:&'a mut Client, name: &'b str) -> Krate<'a,'b> {
    Krate { client: client, name: name }
  }

  pub fn downloads(self) -> Result<Vec<Download>> {
    let body = try!(self.client.get(format!("/crates/{}/downloads", self.name)));
    Ok(json::decode::<MetaDownloads>(&body).unwrap().meta.extra_downloads)
  }

  pub fn follow(self) -> Result<()> {
    let body = try!(self.client.put(format!("/crates/{}/follow", self.name), None));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  pub fn unfollow(self) -> Result<()> {
    let body = try!(self.client.delete(format!("/crates/{}/follow", self.name), None));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  pub fn following(self) -> Result<bool> {
    let body = try!(self.client.get(format!("/crates/{}/following", self.name)));
    Ok(json::decode::<Following>(&body).unwrap().following)
  }

  pub fn get(self) -> Result<Crate> {
    let body = try!(self.client.get(format!("/crates/{}", self.name)));
    Ok(json::decode::<CrateReq>(&body).unwrap().krate)
  }

  pub fn owners(self) -> Result<Vec<User>> {
    let body = try!(self.client.get(format!("/crates/{}/owners", self.name)));
    Ok(json::decode::<Users>(&body).unwrap().users)
  }

  pub fn add_owners(self, owners: &[&str]) -> Result<()> {
    let data = json::encode(&OwnersReq { users: owners }).unwrap();
    let mut bytes = data.as_bytes();
    let body = try!(self.client.put(format!("/crates/{}/owners", self.name),
                             Some(Body::new(&mut Box::new(&mut bytes), bytes.len() as u64))));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  pub fn remove_owners(self, owners: &[&str]) -> Result<()> {
    let data = json::encode(&OwnersReq { users: owners }).unwrap();
    let mut bytes = data.as_bytes();
    let body = try!(self.client.delete(format!("/crates/{}/owners", self.name),
                                Some(Body::new(&mut Box::new(&mut bytes), bytes.len() as u64))));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  pub fn reverse_dependencies(&mut self) -> Result<Vec<Dependency>> {
    let body = try!(self.client.get(format!("/crates/{}/reverse_dependencies", self.name)));
    Ok(json::decode::<Dependencies>(&body).unwrap().dependencies)
  }

  pub fn version<'c>(&'c mut self, version: &'c str) -> KrateVersion {
    KrateVersion::new(self.client, self.name, version)
  }

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

  pub fn krate<'a>(&'a mut self, name: &'a str) -> Krate {
    Krate::new(self, name)
  }

  // todo: sort (downloads|name), by letter/keyword/user_id/following
  pub fn find(&mut self, query: &str) -> Result<Vec<Crate>> {
    let body = try!(self.get(format!("/crates?q={}&sort={}", query, "name")));
    Ok(json::decode::<Crates>(&body).unwrap().crates)
  }

  // todo: publish -- https://github.com/rust-lang/crates.io/blob/dabd8778c1a515ea7572c59096da76e562afe2e2/src/lib.rs#L76
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
  use test::Bencher;

  #[test]
  fn it_parses_crate_req() {

  }

  #[bench]
  fn it_benches(b: &mut Bencher) {

  }
}
