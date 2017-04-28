// Apparently this is recommended when using error-chain.
#![recursion_limit = "1024"]

// TODO: Remove before stable release.
#![allow(dead_code)]

#[macro_use]
extern crate error_chain;
extern crate hyper;
extern crate uuid;
extern crate sxd_document;
extern crate sxd_xpath;

pub mod errors {
    pub type XmlParserErrors = (usize, ::std::vec::Vec<::sxd_document::parser::Error>);

    error_chain!{
        types {
            ParseError, ParseErrorKind, ChainParseErr;
        }

        // Automatic conversions between this error chain and errors not defined using error
        // chain.
        foreign_links {
            XmlParserError(::sxd_document::parser::Error);
            XpathError(::sxd_xpath::Error);
            XpathExecutionError(::sxd_xpath::ExecutionError);
            XpathParserError(::sxd_xpath::ParserError);
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

    impl From<(usize, ::std::vec::Vec<::sxd_document::parser::Error>)> for ParseError {
        fn from(err: (usize, ::std::vec::Vec<::sxd_document::parser::Error>)) -> ParseError {
            ParseErrorKind::XmlParserError(err.1[0]).into()
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

pub mod client;
pub mod entities;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
