use uuid;
use sxd_document;
use sxd_document::parser::parse as sxd_parse;
use sxd_xpath;
use sxd_xpath::evaluate_xpath;

use std::io::Read;

/// Identifier for entities in the MusicBrainz database.
/// TODO: Figure out if it makes more sense to keep
pub type Mbid = uuid::Uuid;

pub trait Resource {
    // TODO: add inc= support
    fn lookup(mbid: &Mbid, inc: Option<()>);

    //    pub
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AreaType {
    /// Areas included (or previously included) in ISO 3166-1.
    Country,

    /// Main administrative divisions of a country.
    Subdivision,

    /// Smaller administrative divisions of a country, which are not one of the main administrative
    /// divisions but are also not muncipalities.
    County,

    /// Small administrative divisions. Urban muncipalities often contain only a single city and a
    /// few surrounding villages, while rural muncipalities often group several villages together.
    Muncipality,

    /// Settlements of any size, including towns and villages.
    City,

    /// Used for a division of a large city.
    District,

    /// Islands and atolls which don't form subdivisions of their own.
    Island,
}

/// A geographic region or settlement.
/// This is one of the `core entities` of MusicBrainz.
pub struct Area {
    /// MBID of the entity in the MusicBrainz database.
    pub mbid: Mbid,

    /// The name of the area.
    pub name: String,

    /// The type of the area.
    pub area_type: AreaType,

    /// ISO 3166 code, assigned to countries and subdivisions.
    pub iso_3166: Option<String>,
}

pub type SxdParserError = sxd_document::parser::Error;
type SxdParserErrors = (usize, Vec<sxd_document::parser::Error>);
pub type SxdXpathError = sxd_xpath::Error;

#[derive(Debug)]
pub enum ReadError {
    XmlParserError(SxdParserError),
    XmlXpathError(SxdXpathError),
    InvalidData(String),
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

struct XpathReader<'d> {
    context: sxd_xpath::Context<'d>,
    factory: sxd_xpath::Factory,
}

impl<'d> XpathReader<'d> {
    fn new() -> Result<XpathReader<'d>, ReadError> {
        let mut context = sxd_xpath::Context::<'d>::default();
        context.set_namespace("mb", "http://musicbrainz.org/ns/mmd-2.0#");

        Ok(XpathReader {
            context: context,
            factory: sxd_xpath::Factory::default(),
        })
    }

    fn evaluate<'a, N>(&self,
                       node: N,
                       xpath_query: &'a str)
                       -> Result<sxd_xpath::Value<'d>, ReadError>
        where N: Into<sxd_xpath::nodeset::Node<'d>>
    {
        // let node = node.into();
        // println!("prefixed name: {:?}", node.prefixed_name());
        // println!("expanded name: {:?}", node.expanded_name());
        // println!("children: {:?}", node.children());

        // TODO remove unwrap.
        let xpath = self.factory
            .build(xpath_query)?
            .unwrap();
        xpath.evaluate(&self.context, node.into()).map_err(|err| ReadError::from(err))
    }
}

impl Area {
    pub fn read_xml<'a>(xml: &'a str) -> Result<Area, ReadError> {
        let reader = XpathReader::new()?;

        let package = sxd_parse(xml)?;
        let document = package.as_document();

        let mbid = Mbid::parse_str(&reader.evaluate(document.root(), "//mb:area/@id")?
            .string()[..])?;

        let area_type = match reader.evaluate(document.root(), "//mb:area/@type")?
            .string()
            .as_ref() {
            "Country" => AreaType::Country,
            "Subdivision" => AreaType::Subdivision,
            "County" => AreaType::County,
            "Muncipality" => AreaType::Muncipality,
            "City" => AreaType::City,
            "District" => AreaType::District,
            "Island" => AreaType::Island,
            s => {
                return Err(ReadError::InvalidData(format!("Unknown area type: {}", s).to_string()))
            }
        };
        let name = reader.evaluate(document.root(), "//mb:area/mb:name/text()")?.string();
        let iso_3166_str = reader.evaluate(document.root(),
                      "//mb:area/mb:iso-3166-1-code-list/mb:iso-3166-1-code/text()")?
            .string();

        Ok(Area {
            mbid: mbid,
            name: name,
            area_type: area_type,
            iso_3166: if iso_3166_str.is_empty() {
                None
            } else {
                Some(iso_3166_str)
            }
        })
    }
}

pub struct Artist {}

pub struct Event {}

pub struct Instrument {}

pub struct Label {}

pub struct Recording {}

pub struct Release {}

pub struct ReleaseGroup {}

pub struct Series {}

pub struct Work {}

pub struct Url {}

// TODO: rating, tag, collection
// TODO: discid, isrc, iswc

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_read_xml1() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
                    <metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#">
                        <area id="a1411661-be21-4290-8dc1-50f3d8e3ea67" type="City" type-id="6fd8f29a-3d0a-32fc-980d-ea697b69da78">
                            <name>Honolulu</name>
                            <sort-name>Honolulu</sort-name>
                        </area>
                    </metadata>"#;
        let result = Area::read_xml(&xml).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("a1411661-be21-4290-8dc1-50f3d8e3ea67").unwrap());
        assert_eq!(result.name, "Honolulu".to_string());
        assert_eq!(result.area_type, AreaType::City);
        assert_eq!(result.iso_3166, None);
    }

    #[test]
    fn area_read_xml2() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><area type-id="06dd0ae4-8c74-30bb-b43d-95dcedf961de" type="Country" id="2db42837-c832-3c27-b4a3-08198f75693c"><name>Japan</name><sort-name>Japan</sort-name><iso-3166-1-code-list><iso-3166-1-code>JP</iso-3166-1-code></iso-3166-1-code-list></area></metadata>"#;
        let result = Area::read_xml(&xml).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("2db42837-c832-3c27-b4a3-08198f75693c").unwrap());
        assert_eq!(result.name, "Japan".to_string());
        assert_eq!(result.area_type, AreaType::Country);
        assert_eq!(result.iso_3166, Some("JP".to_string()));
    }
}
