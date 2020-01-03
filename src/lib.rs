#[macro_use]
extern crate lazy_static;

extern crate bottle;
extern crate http;
extern crate rayon;

mod proxy;

pub use proxy::{generic_proxy, FF_PROXT_HOST};
