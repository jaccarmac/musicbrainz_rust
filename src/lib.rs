// Apparently this is recommended when using error-chain.
#![recursion_limit = "1024"]

// TODO: Remove before stable release.
#![allow(dead_code)]

#[macro_use]
extern crate error_chain;
extern crate hyper;
extern crate uuid;
extern crate xpath_reader;

pub mod errors {
    error_chain!{
        types {
            ParseError, ParseErrorKind, ChainParseErr;
        }

        links {
            XpathReadError(::xpath_reader::XpathError, ::xpath_reader::XpathErrorKind);
        }

        // Automatic conversions between this error chain and errors not defined using error
        // chain.
        foreign_links {
            UuidParseError(::uuid::ParseError);
            ParseIntError(::std::num::ParseIntError);
            ParseDateError(super::entities::ParseDateError);
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

    error_chain!{
        types {
            ClientError, ClientErrorKind, ChainClientErr;
        }

        links {
            ParseError(ParseError, ParseErrorKind);
        }

        foreign_links {
            HttpError(::hyper::error::Error);
            HyperParserError(::hyper::error::ParseError);
            IoError(::std::io::Error);
        }
    }
}
pub use errors::*;

//pub mod client;
pub mod entities;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
