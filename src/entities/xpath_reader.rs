use sxd_xpath;
use sxd_xpath::{Context, Factory, Value, XPath};
use sxd_xpath::nodeset::Node;
use sxd_document;
use sxd_document::Package;
use sxd_document::parser::parse as sxd_parse;

use super::Mbid;

pub fn default_musicbrainz_context<'d>() -> Context<'d> {
    let mut context = Context::<'d>::default();
    context.set_namespace("mb", "http://musicbrainz.org/ns/mmd-2.0#");
    context
}

/// Allows to execute XPath expressions on some kind of abstract document structure.
pub trait XPathReader<'d> {
    /// Evaluate an XPath expression on the root of this reader.
    fn evaluate(&'d self, xpath_expr: &str) -> Result<Value<'d>, ReadError>;

    /// Evaluate an XPath expression, parsing the result into a Mbid.
    fn read_mbid(&'d self, xpath_expr: &str) -> Result<Mbid, ReadError> {
        Ok(Mbid::parse_str(&self.evaluate(xpath_expr)?.string()[..])?)
    }
}

/// Reader that parses an XML string and runs expressions against the root element.
pub struct XPathStrReader<'d> {
    context: Context<'d>,
    factory: Factory,
    package: Package,
}

/// Reader that takes another node as input and allows parsing against this node as root.
pub struct XPathNodeReader<'d> {
    factory: Factory,
    node: Node<'d>,
    context: &'d Context<'d>,
}

// TODO
pub type SxdParserError = sxd_document::parser::Error;
type SxdParserErrors = (usize, Vec<sxd_document::parser::Error>);
pub type SxdXpathError = sxd_xpath::Error;

#[derive(Debug)]
pub enum ReadError {
    XmlParserError(SxdParserError),
    XmlXpathError(SxdXpathError),
    InvalidData(String),
    /// There was an internal error somewhere in our code. If this occurs it is considered a bug
    /// that should be reported and fixed.
    InternalError(String),
}

fn build_xpath(factory: &Factory, xpath_expr: &str) -> Result<XPath, ReadError> {
    factory.build(xpath_expr)?
        .ok_or(ReadError::InternalError("XPath instance was None!".to_string()))
}

impl<'d> XPathStrReader<'d> {
    pub fn new(xml: &str) -> Result<Self, ReadError> {

        Ok(Self {
            context: default_musicbrainz_context(),
            factory: Factory::default(),
            package: sxd_parse(xml)?,
        })
    }
}

impl<'d> XPathReader<'d> for XPathStrReader<'d> {
    fn evaluate(&'d self, xpath_expr: &str) -> Result<Value<'d>, ReadError> {
        let xpath = build_xpath(&self.factory, xpath_expr)?;
        xpath.evaluate(&self.context, self.package.as_document().root())
            .map_err(|err| ReadError::from(err))
    }
}

impl<'d> XPathNodeReader<'d> {
    pub fn new<N>(node: N, context: &'d Context<'d>) -> Result<Self, ReadError>
        where N: Into<Node<'d>>
    {
        Ok(Self {
            node: node.into(),
            factory: Factory::default(),
            context: context,
        })
    }
}

impl<'d> XPathReader<'d> for XPathNodeReader<'d> {
    fn evaluate(&'d self, xpath_expr: &str) -> Result<Value<'d>, ReadError> {
        let xpath = build_xpath(&self.factory, xpath_expr)?;
        xpath.evaluate(self.context, self.node).map_err(|err| ReadError::from(err))
    }
}

impl From<SxdParserError> for ReadError {
    fn from(e: SxdParserError) -> ReadError {
        ReadError::XmlParserError(e)
    }
}

impl From<SxdParserErrors> for ReadError {
    fn from(e: SxdParserErrors) -> ReadError {
        ReadError::XmlParserError(e.1[0])
    }
}

impl From<SxdXpathError> for ReadError {
    fn from(e: SxdXpathError) -> ReadError {
        ReadError::XmlXpathError(e)
    }
}

impl From<::sxd_xpath::ParserError> for ReadError {
    fn from(e: ::sxd_xpath::ParserError) -> ReadError {
        ReadError::XmlXpathError(::sxd_xpath::Error::Parsing(e))
    }
}

impl From<::sxd_xpath::ExecutionError> for ReadError {
    fn from(e: ::sxd_xpath::ExecutionError) -> ReadError {
        ReadError::XmlXpathError(::sxd_xpath::Error::Executing(e))
    }
}

impl From<::uuid::ParseError> for ReadError {
    fn from(err: ::uuid::ParseError) -> ReadError {
        ReadError::InvalidData(format!("Failed parsing string as uuid: {}", err).to_string())
    }
}
