// #[macro_use]
extern crate hyper;

mod proxy;

pub use proxy::{generic_proxy, simple_proxy};
