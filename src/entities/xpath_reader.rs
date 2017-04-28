// TODO: this should probably be moved to a different file/directory

use sxd_xpath::{Context, Factory, Value, XPath};
use sxd_xpath::nodeset::Node;
use sxd_xpath::Value::Nodeset;
use sxd_document::Package;
use sxd_document::parser::parse as sxd_parse;

use super::{Date, Mbid, ParseError, ParseErrorKind, non_empty_string};

pub fn default_musicbrainz_context<'d>() -> Context<'d> {
    let mut context = Context::<'d>::default();
    context.set_namespace("mb", "http://musicbrainz.org/ns/mmd-2.0#");
    context
}

/// A trait to abstract the idea of something that can be parsed from XML.
pub trait FromXml
    where Self: Sized
{
    /// Read an instance of `Self` from the provided `reader`.
    ///
    /// The reader can be relative to a specific element. Whether the root of the document contains
    /// the element to be parsed or is the element to be parsed can be specified by the additional
    /// traits `FromXmlContained` and `FromXmlElement`.
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ParseError> where R: XPathReader<'d>;
}

/// `FromXml` takes a reader as input whose root element **contains** the relevant element.
pub trait FromXmlContained: FromXml {}
/// `FromXml` takes a reader as input whose root element **is** the relevant element.
pub trait FromXmlElement: FromXml {}

/// Allows to execute XPath expressions on some kind of abstract document structure.
pub trait XPathReader<'d> {
    /// Evaluate an XPath expression on the root of this reader.
    fn evaluate(&'d self, xpath_expr: &str) -> Result<Value<'d>, ParseError>;

    /// Return a reference to the context of the reader.
    fn context(&'d self) -> &'d Context<'d>;

    /// Evaluate an XPath expression, parsing the result into a `Mbid`.
    fn read_mbid(&'d self, xpath_expr: &str) -> Result<Mbid, ParseError> {
        Ok(Mbid::parse_str(&self.evaluate(xpath_expr)?.string()[..])?)
    }

    /// Evaluate an XPath expression, parsing the result into a `String`.
    fn read_string(&'d self, xpath_expr: &str) -> Result<String, ParseError> {
        Ok(self.evaluate(xpath_expr)?.string())
    }

    /// Evaluate an XPath expression, parsing the result into an `Option<String>` which is `None`
    /// when it would be parsed into an empty string otherwise.
    fn read_nstring(&'d self, xpath_expr: &str) -> Result<Option<String>, ParseError> {
        Ok(non_empty_string(self.evaluate(xpath_expr)?.string()))
    }

    /// Evaluate an XPath expression, parsing the result into a `Date`.
    fn read_date(&'d self, xpath_expr: &str) -> Result<Date, ParseError> {
        Ok(self.evaluate(xpath_expr)?.string().parse()?)
    }

    /// Read the nodeset specified by the `xpath_expr`, which can be anything which can be deserialized from XML,
    /// into a `Vec`. If something other than a nodeset or nothing is found an empty vector will be returned.
    fn read_vec<Item>(&'d self, xpath_expr: &str) -> Result<Vec<Item>, ParseError>
        where Item: FromXmlElement
    {
        match self.evaluate(xpath_expr)? {
            Nodeset(nodeset) => {
                let context = default_musicbrainz_context();
                nodeset.document_order()
                    .iter()
                    .map(|node| {
                             XPathNodeReader::new(*node, &context).and_then(|r| Item::from_xml(&r))
                         })
                    .collect()
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Evaluates an XPath query, takes the first returned node (in document order) and creates
    /// a new XPathNodeReader with that node.
    fn relative_reader(&'d self, xpath_expr: &str) -> Result<XPathNodeReader<'d>, ParseError> {
        let node: Node<'d> = match self.evaluate(xpath_expr)? {
            Value::Nodeset(nodeset) => {
                let res: Result<Node<'d>, ParseError> = nodeset
                    .document_order_first()
                    // TODO consider better error to return.
                    .ok_or_else(||
                        ParseErrorKind::InvalidData(format!("failed to find a node with the specified xpath '{}'", xpath_expr)).into());
                res?
            }
            _ => return Err(ParseErrorKind::InvalidData(format!("xpath didn't specify a nodeset: '{}'", xpath_expr)).into()),
        };
        XPathNodeReader::new(node, self.context())
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

fn build_xpath(factory: &Factory, xpath_expr: &str) -> Result<XPath, ParseError> {
    factory
        .build(xpath_expr)?
        .ok_or_else(|| ParseErrorKind::InternalError("XPath instance was `None`!".to_string()).into())
}

impl<'d> XPathStrReader<'d> {
    // TODO: Once the rest of the code stabilizes consider taking a reference to a context, so it
    // doesn't have to be created anew for each reader instance.
    pub fn new(xml: &str) -> Result<Self, ParseError> {
        Ok(Self {
               context: default_musicbrainz_context(),
               factory: Factory::default(),
               package: sxd_parse(xml)?,
           })
    }
}

impl<'d> XPathReader<'d> for XPathStrReader<'d> {
    fn evaluate(&'d self, xpath_expr: &str) -> Result<Value<'d>, ParseError> {
        let xpath = build_xpath(&self.factory, xpath_expr)?;
        xpath.evaluate(&self.context, self.package.as_document().root()).map_err(ParseError::from)
    }

    fn context(&'d self) -> &'d Context<'d> {
        &self.context
    }
}

impl<'d> XPathNodeReader<'d> {
    pub fn new<N>(node: N, context: &'d Context<'d>) -> Result<Self, ParseError>
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
    fn evaluate(&'d self, xpath_expr: &str) -> Result<Value<'d>, ParseError> {
        let xpath = build_xpath(&self.factory, xpath_expr)?;
        xpath.evaluate(self.context, self.node).map_err(ParseError::from)
    }

    fn context(&'d self) -> &'d Context<'d> {
        self.context
    }
}

impl FromXmlElement for String {}
impl FromXml for String {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ParseError> where R: XPathReader<'d> {
        reader.read_string(".")
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
