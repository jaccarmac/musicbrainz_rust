// TODO: this should probably be moved to a different file/directory

use sxd_xpath;
use sxd_xpath::{Context, Factory, Value, XPath};
use sxd_xpath::nodeset::Node;
use sxd_document;
use sxd_document::Package;
use sxd_document::parser::parse as sxd_parse;

use super::{Mbid, ReadError, ReadErrorKind};

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

fn build_xpath(factory: &Factory, xpath_expr: &str) -> Result<XPath, ReadError> {
    factory
        .build(xpath_expr)?
        .ok_or_else(|| ReadErrorKind::InternalError("XPath instance was `None`!".to_string()).into())
}

impl<'d> XPathStrReader<'d> {
    // TODO: Once the rest of the code stabilizes consider taking a reference to a context, so it
    // doesn't have to be created anew for each reader instance.
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
        xpath.evaluate(&self.context, self.package.as_document().root()).map_err(ReadError::from)
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
        xpath.evaluate(self.context, self.node).map_err(ReadError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC_1: &'static str =
        r#"<?xml version="1.0" encoding="UTF-8"?><root><child name="Hello World"/></root>"#;

    #[test]
    fn xpath_str_reader() {
        let reader = XPathStrReader::new(DOC_1).unwrap();
        assert_eq!(reader.evaluate(".//child/@name").unwrap().string(),
                   "Hello World".to_string());
    }

    #[test]
    fn xpath_node_reader() {
        use sxd_xpath::Value::Nodeset;

        let str_reader = XPathStrReader::new(DOC_1).unwrap();
        match str_reader.evaluate(".//child").unwrap() {
            Nodeset(nodeset) => {
                let node = nodeset.document_order_first().unwrap();
                let context = default_musicbrainz_context();
                let node_reader = XPathNodeReader::new(node, &context).unwrap();

                assert_eq!(node_reader.evaluate(".//@name").unwrap().string(),
                           "Hello World".to_string());
            }
            _ => panic!("Nodeset not found!"),
        };

    }
}

