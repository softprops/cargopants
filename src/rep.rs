//! Representation of various crates.io entities
extern crate rustc_serialize;

use std::collections::HashMap;

/// Representation of a crates.io User
#[derive(RustcDecodable)]
#[derive(Debug)]
pub struct User {
  /// User id
  pub id: u32,
  /// User login name
  pub login: String,
  /// User avatar url
  pub avatar: String,
  /// User email where available
  pub email: Option<String>,
  /// User name where available
  pub name: Option<String>,
}

/// Representation of a crates.io Crate
#[derive(RustcDecodable)]
#[derive(Debug)]
pub struct Crate {
  /// name of crate
  pub name: String,
  /// description of create
  pub description: Option<String>,
  /// the most recent version
  pub max_version: String
}

/// Representation of the downloads of a version on a given date
#[derive(RustcDecodable)]
#[derive(Debug)]
pub struct Download {
  /// date of downloads
  pub date: String,
  /// number of downloads on that day
  pub downloads: u32
}

/// Representation of a crate dependency
#[derive(RustcDecodable)]
#[derive(Debug)]
pub struct Dependency {
  /// Crate identifier
  pub crate_id: String,
  /// feature defaults
  pub default_features: bool,
  /// features
  pub features: String,
  /// kind of dependency
  pub kind: String,
  /// optional dependency indicator
  pub optional: bool,
  /// ...
  pub req: String  
}

/// Representation of a crate version
#[derive(RustcDecodable)]
#[derive(Debug)]
pub struct Version {
  /// crate: String,
  pub created_at: String,
  /// path to download version
  pub dl_path: String,
  /// number of downloads
  pub downloads: u32,
  /// version number
  pub num: String,
  /// last time this version was updated
  pub updated_at: String,
  /// indicates if version was yanked from crate server
  pub yanked: bool
}

/// Interface for creating a new crate
#[derive(RustcEncodable)]
pub struct NewCrate {
  /// name of crate
  pub name: String,
  /// version to post
  pub vers: String,
  /// vector of dependencies
  pub deps: Vec<NewCrateDependency>,
  /// features
  pub features: HashMap<String, Vec<String>>,
  /// vector of authors
  pub authors: Vec<String>,
  /// description of crate
  pub description: Option<String>,
  /// link to documentation
  pub documentation: Option<String>,
  /// link to homepage
  pub homepage: Option<String>,
  /// link to readme
  pub readme: Option<String>,
  /// vector of keywords
  pub keywords: Vec<String>,
  /// name of license
  pub license: Option<String>,
  /// path of licence file
  pub license_file: Option<String>,
  /// repository of crate
  pub repository: Option<String>,
}

/// Representation of a new crate dependency
#[derive(RustcEncodable)]
pub struct NewCrateDependency {
  /// optional indicator
  pub optional: bool,
  /// feature defaults
  pub default_features: bool,
  /// name of dependency
  pub name: String,
  /// vector of features
  pub features: Vec<String>,
  /// version requirement
  pub version_req: String,
  /// ...
  pub target: Option<String>,
  /// kind of dependency
  pub kind: String,
}
