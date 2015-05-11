extern crate hyper;

use hyper::{ client, Url };
use hyper::method::Method;
use hyper::header::{
  Accept, Authorization, ContentType,
  UserAgent, qitem
};
use mime::{ Attr, Mime, Value };
use mime::TopLevel::Application;
use mime::SubLevel::Json;
use std::io::prelude::*;
use std::io::Result;
use std::ops::DerefMut;

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
