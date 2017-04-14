use uuid;
use sxd_document;
use sxd_document::parser::parse as sxd_parse;
use sxd_xpath;
use sxd_xpath::evaluate_xpath;

use std::io::Read;
use std::str::FromStr;

mod xpath_reader;
use self::xpath_reader::*;
pub use self::xpath_reader::{ReadError, SxdParserError, SxdXpathError};

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

    /// Main administrative divisions of a countryr
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
///
/// https://musicbrainz.org/doc/Area
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


impl Area {
    fn read_xml<'d, R>(reader: &'d R) -> Result<Area, ReadError>
        where R: XPathReader<'d>
    {
        let mbid = reader.read_mbid("//mb:area/@id")?;

        let area_type = match reader.evaluate("//mb:area/@type")?
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
        let name = reader.evaluate("//mb:area/mb:name/text()")?.string();
        let iso_3166_str =
            reader.evaluate("//mb:area/mb:iso-3166-1-code-list/mb:iso-3166-1-code/text()")?
                .string();

        Ok(Area {
            mbid: mbid,
            name: name,
            area_type: area_type,
            iso_3166: if iso_3166_str.is_empty() {
                None
            } else {
                Some(iso_3166_str)
            },
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ArtistType {
    Person,
    Group,
    Orchestra,
    Choir,
    Character,
    Other,
}

/// TODO: Find all possible variants. (It says "male, female or neither" in the docs but what does
/// this mean. Is there a difference between unknown genders and non-binary genders?)
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Gender {
    Female,
    Male,
    Other(String),
}

/// https://musicbrainz.org/doc/Artist
pub struct Artist {
    /// MBID of the entity in the MusicBrainz database.
    pub mbid: Mbid,

    /// The official name of the artist.
    pub name: String,

    /// Name to properly sort the artist by.
    /// Even for artists whose `name` is written in a different script this one will be in latin
    /// script. The full [guidelines](https://musicbrainz.org/doc/Style/Artist/Sort_Name) are a bit more complicated.
    pub sort_name: String,

    /// Whether this Artist is a person, group, or something else.
    pub artist_type: ArtistType,

    /// If the Artist is a single person this indicates their gender.
    pub gender: Option<Gender>,

    /// TODO: Is this an actual area or just an Id? TODO represent what we get from the api and
    /// if it's just an ID provide methods to fetch these.
    pub area: Area,

    // TODO: begin and end dates
    pub ipi_code: Option<String>,
    pub isni_code: Option<String>, /* TODO aliases
                                    * TODO disambiguation comment */
}

impl Artist {
    fn read_xml(xml: &str) -> Result<Self, ReadError> {
        let reader = XPathStrReader::new(xml)?;

        let mbid = reader.read_mbid("//mb:artist/@id")?;
        let name = reader.evaluate("//mb:artist/mb:name/text()")?.string();
        let sort_name = reader.evaluate("//mb:artist/mb:sort-name/text()")?
            .string();
        let artist_type = match reader.evaluate("//mb:artist/@type")?.string().as_ref() {
            "Person" => ArtistType::Person,
            "Group" => ArtistType::Group,
            "Orchestra" => ArtistType::Orchestra,
            "Choir" => ArtistType::Choir,
            "Character" => ArtistType::Character,
            "Other" => ArtistType::Other,
            t => {
                return Err(ReadError::InvalidData(format!("Unknown artist type: {}", t)
                    .to_string()))
            }
        };

        // Get area information.
        let area_val = match reader.evaluate("//mb:artist/mb:area")? {
            sxd_xpath::Value::Nodeset(nodeset) => {
                if let Some(node) = nodeset.document_order_first() {

                    // Extract Area struct from the node.
                    // TODO
                } else {
                    return Err(ReadError::InvalidData("Area element is empty.".to_string()));
                }
            }
            _ => return Err(ReadError::InvalidData("Area value is not a nodeset.".to_string())),
        };


        Err(ReadError::InvalidData("TODO".to_string()))
    }
}

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
        let reader = XPathStrReader::new(xml).unwrap();
        let result = Area::read_xml(&reader).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("a1411661-be21-4290-8dc1-50f3d8e3ea67").unwrap());
        assert_eq!(result.name, "Honolulu".to_string());
        assert_eq!(result.area_type, AreaType::City);
        assert_eq!(result.iso_3166, None);
    }

    #[test]
    fn area_read_xml2() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><area type-id="06dd0ae4-8c74-30bb-b43d-95dcedf961de" type="Country" id="2db42837-c832-3c27-b4a3-08198f75693c"><name>Japan</name><sort-name>Japan</sort-name><iso-3166-1-code-list><iso-3166-1-code>JP</iso-3166-1-code></iso-3166-1-code-list></area></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let result = Area::read_xml(&reader).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("2db42837-c832-3c27-b4a3-08198f75693c").unwrap());
        assert_eq!(result.name, "Japan".to_string());
        assert_eq!(result.area_type, AreaType::Country);
        assert_eq!(result.iso_3166, Some("JP".to_string()));
    }

    #[test]
    fn artist_read_xml1() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><artist id="90e7c2f9-273b-4d6c-a662-ab2d73ea4b8e" type-id="e431f5f6-b5d2-343d-8b36-72607fffb74b" type="Group"><name>NECRONOMIDOL</name><sort-name>NECRONOMIDOL</sort-name><country>JP</country><area id="2db42837-c832-3c27-b4a3-08198f75693c"><name>Japan</name><sort-name>Japan</sort-name><iso-3166-1-code-list><iso-3166-1-code>JP</iso-3166-1-code></iso-3166-1-code-list></area><begin-area id="8dc97297-ac95-4d33-82bc-e07fab26fb5f"><name>Tokyo</name><sort-name>Tokyo</sort-name><iso-3166-2-code-list><iso-3166-2-code>JP-13</iso-3166-2-code></iso-3166-2-code-list></begin-area><life-span><begin>2014-03</begin></life-span></artist></metadata><Paste>"#;
        let result = Artist::read_xml(&xml).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("90e7c2f9-273b-4d6c-a662-ab2d73ea4b8e").unwrap());
        assert_eq!(result.name, "NECRONOMIDOL".to_string());
        assert_eq!(result.sort_name, "NECRONOMIDOL".to_string());
        // TODO: Check area.
        assert_eq!(result.artist_type, ArtistType::Group);
        assert_eq!(result.gender, None);
        assert_eq!(result.ipi_code, None);
        assert_eq!(result.isni_code, None);
    }
}
