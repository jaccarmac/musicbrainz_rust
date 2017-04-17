// Apparently this is recommended when using error-chain.
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate uuid;
extern crate sxd_document;
extern crate sxd_xpath;

pub mod errors {
    error_chain!{
        types {
            ReadError, ReadErrorKind, ChainReadErr, ReadResult;
        }

        // Automatic conversions between this error chain and errors not defined using error
        // chain.
        foreign_links {
            XmlParserError(::sxd_document::parser::Error);
            XmlXpathError(::sxd_xpath::Error);
        }

        // Custom error kinds.
        errors {
            InvalidData(msg: String) {
                description("invalid data")
                display("invalid data: {}", msg)
            }
            /// Somewhere in our code something went wrong, that really shouldn't have.
            /// These are always considered a bug that should reported as an issue.
            InternalError(msg: String) {
                description("internal error")
                display("internal error: {}\nYou should probably report this bug.", msg)
            }
        }
    }
}
//use errors::*;

pub mod entities;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
