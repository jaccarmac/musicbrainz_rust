// Apparently this is recommended when using error-chain.
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate uuid;
extern crate sxd_document;
extern crate sxd_xpath;

pub mod errors {
    error_chain! {

    }
}
use errors::*;

pub mod entities;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
