extern crate flask;
extern crate http;
extern crate num_cpus;
extern crate rayon;

mod proxy;

pub use proxy::generic_proxy;
pub use proxy::simple_proxy;
