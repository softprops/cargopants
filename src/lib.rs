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
use rustc_serialize::{ Decoder, Decodable, json };
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
struct Following {
  following: bool
}

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
 
  // todo: soft (downloads|name), by letter/keyword/user_id/following
  pub fn find(&mut self, query: &str) -> Result<Vec<Crate>> {
    let body = try!(self.get(format!("/crates?q={}&sort={}", query, "name")));
    Ok(json::decode::<Crates>(&body).unwrap().crates)
  }

  pub fn named(&mut self, name: &str) -> Result<Crate> { // can't decode this automagically because it contains the key "crate"!
    let body = try!(self.get(format!("/crates/{}", name)));
    Ok(json::decode::<CrateReq>(&body).unwrap().krate)
  }

  // todo: publish -- https://github.com/rust-lang/crates.io/blob/dabd8778c1a515ea7572c59096da76e562afe2e2/src/lib.rs#L76

  pub fn version(&mut self, name: &str, version: &str) -> Result<Version> {
    let body = try!(self.get(format!("/crates/{}/{}", name, version)));
    Ok(json::decode::<VersionReq>(&body).unwrap().version)
  }

  // todo: version download -- https://github.com/rust-lang/crates.io/blob/dabd8778c1a515ea7572c59096da76e562afe2e2/src/lib.rs#L78

  pub fn dependencies(&mut self, name: &str, version: &str) -> Result<Vec<Dependency>> {
    let body = try!(self.get(format!("/crates/{}/{}/dependencies", name, version)));
    Ok(json::decode::<Dependencies>(&body).unwrap().dependencies)
  }

  pub fn downloads(&mut self, name: &str, version: &str) -> Result<Vec<Download>> {
    let body = try!(self.get(format!("/crates/{}/{}/downloads", name, version)));
    Ok(json::decode::<VersionDownloads>(&body).unwrap().version_downloads)
  }

  pub fn authors(&mut self, name: &str, version: &str) -> Result<Vec<String>> {
    let body = try!(self.get(format!("/crates/{}/{}/authors", name, version)));
    Ok(json::decode::<Authors>(&body).unwrap().meta.names)
  }

  pub fn all_downloads(&mut self, name: &str) -> Result<Vec<Download>> {
    let body = try!(self.get(format!("/crates/{}/downloads", name)));
    Ok(json::decode::<MetaDownloads>(&body).unwrap().meta.extra_downloads)
  }

  pub fn versions(&mut self, name: &str) -> Result<Vec<Version>> {
    let body = try!(self.get(format!("/crates/{}/versions", name)));
    let versions: Vec<Version> = json::decode::<Versions>(&body).unwrap().versions;
    Ok(versions)
  }

  pub fn follow(&mut self, krate: &str) -> Result<()> {
    let body = try!(self.put(format!("/crates/{}/follow", krate), &vec![]));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  pub fn unfollow(&mut self, krate: &str) -> Result<()> {
    let body = try!(self.delete(format!("/crates/{}/follow", krate), None));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  pub fn following(&mut self, krate: &str) -> Result<bool> {
    let body = try!(self.get(format!("/crates/{}/following", krate)));
    Ok(json::decode::<Following>(&body).unwrap().following)
  }

  pub fn owners(&mut self, krate: &str) -> Result<Vec<User>> {
    let body = try!(self.get(format!("/crates/{}/owners", krate)));
    Ok(json::decode::<Users>(&body).unwrap().users)
  }

  pub fn add_owners(&mut self, krate: &str, owners: &[&str]) -> Result<()> {
    let body = json::encode(&OwnersReq { users: owners }).unwrap();
    let body = try!(self.put(format!("/crates/{}/owners", krate),
                   body.as_bytes()));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  pub fn remove_owners(&mut self, krate: &str, owners: &[&str]) -> Result<()> {
    let body = json::encode(&OwnersReq { users: owners }).unwrap();
    let body = try!(self.delete(format!("/crates/{}/owners", krate),
                     Some(body.as_bytes())));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  pub fn yank(&mut self, krate: &str, version: &str) -> Result<()> {
    let body = try!(self.delete(format!("/crates/{}/{}/yank", krate, version), None));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  pub fn unyank(&mut self, krate: &str, version: &str) -> Result<()> {
    let body = try!(self.put(format!("/crates/{}/{}/unyank", krate, version), &vec![]));
    assert!(json::decode::<Status>(&body).unwrap().ok);
    Ok(())
  }

  // todo: reverse deps -- https://github.com/rust-lang/crates.io/blob/dabd8778c1a515ea7572c59096da76e562afe2e2/src/lib.rs#L92

  // todo: all versions -- https://github.com/rust-lang/crates.io/blob/dabd8778c1a515ea7572c59096da76e562afe2e2/src/lib.rs#L93

  // todo: version? -- https://github.com/rust-lang/crates.io/blob/dabd8778c1a515ea7572c59096da76e562afe2e2/src/lib.rs#L94

  // todo: keyboards? -- https://github.com/rust-lang/crates.io/blob/dabd8778c1a515ea7572c59096da76e562afe2e2/src/lib.rs#L95

  // todo: keyboard -- https://github.com/rust-lang/crates.io/blob/dabd8778c1a515ea7572c59096da76e562afe2e2/src/lib.rs#L96

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
