use uuid;
use sxd_xpath::Value::Nodeset;
use std::str::FromStr;
/// TODO consider what type to use
pub use std::time::Duration;

mod xpath_reader;
use self::xpath_reader::*;
use super::{ReadError, ReadErrorKind};

mod date;
pub use self::date::{Date, ParseDateError};

mod refs;
pub use self::refs::{AreaRef, ArtistRef, LabelRef, RecordingRef, ReleaseRef};

mod area;
mod label;
mod recording;
mod release;
mod release_group;
pub use self::area::{Area, AreaType};
pub use self::label::Label;
pub use self::recording::Recording;
pub use self::release::{Release, ReleaseTrack, ReleaseStatus, ReleaseMedium};
pub use self::release_group::{ReleaseGroup, ReleaseGroupType, ReleaseGroupPrimaryType, ReleaseGroupSecondaryType};

/// Identifier for entities in the MusicBrainz database.
pub type Mbid = uuid::Uuid;

/// Takes a string and returns an option only containing the string if it was not empty.
fn non_empty_string(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
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
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError> where R: XPathReader<'d>;
}

/// `FromXml` takes a reader as input whose root element **contains** the relevant element.
pub trait FromXmlContained : FromXml {}
/// `FromXml` takes a reader as input whose root element **is** the relevant element.
pub trait FromXmlElement : FromXml {}

pub trait Resource {
    /// Returns the url where one can get a ressource in the valid format for parsing from.
    fn get_url(mbid: &str) -> String;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ArtistType {
    Person,
    Group,
    Orchestra,
    Choir,
    Character,
    Other,
}

impl FromStr for ArtistType {
    type Err = ReadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Person" => Ok(ArtistType::Person),
            "Group" => Ok(ArtistType::Group),
            "Orchestra" => Ok(ArtistType::Orchestra),
            "Choir" => Ok(ArtistType::Choir),
            "Character" => Ok(ArtistType::Character),
            "Other" => Ok(ArtistType::Other),
            t => {
                return Err(ReadErrorKind::InvalidData(format!("Unknown artist type: {}", t)
                                                      .to_string()).into())
            }
        }
    }
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
/// TODO: Figure out which properties are optional and which ones are always given.
pub struct Artist {
    /// MBID of the entity in the MusicBrainz database.
    pub mbid: Mbid,

    /// The official name of the artist.
    pub name: String,

    /// Name to properly sort the artist by.
    /// Even for artists whose `name` is written in a different script this one will be in latin
    /// script. The full [guidelines](https://musicbrainz.org/doc/Style/Artist/Sort_Name) are a bit more complicated.
    pub sort_name: String,

    /// Aliases of the artist name. These include alternative official spellings, and common
    /// misspellings, versions in different scripts and other variations of the artist name.
    pub aliases: Vec<String>,

    /// Whether this Artist is a person, group, or something else.
    pub artist_type: ArtistType,

    /// If the Artist is a single person this indicates their gender.
    pub gender: Option<Gender>,

    /// The area an artist is primarily identified with. Often, but not always, birth/formation
    /// country of the artist/group.
    pub area: Option<AreaRef>,

    // TODO docs
    pub begin_date: Option<Date>,
    // TODO docs
    pub end_date: Option<Date>,

    // TODO docs
    pub ipi_code: Option<String>,
    // TODO docs
    pub isni_code: Option<String>,
                                    /* TODO disambiguation comment */
}

impl FromXml for Artist {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        // Get gender.
        let gender = match reader.evaluate(".//mb:artist/mb:gender/text()") {
            Ok(value) => {
                match value.string().as_ref() {
                    "Female" => Some(Gender::Female),
                    "Male" => Some(Gender::Male),
                    "" => None,
                    other => Some(Gender::Other(other.to_string())),
                }
            }
            _ => None,
        };

        let area = match reader.evaluate(".//mb:artist") {
            Ok(Nodeset(nodeset)) => {
                if let Some(node) = nodeset.document_order_first() {
                    let context = default_musicbrainz_context();
                    let reader = XPathNodeReader::new(node, &context)?;
                    Some(AreaRef::from_xml(&reader)?)
                } else {
                    None
                }
            }
            _ => None,
        };

        let aliases: Vec<String> =
            match reader.evaluate(".//mb:artist/mb:alias-list/mb:alias/text()")? {
                Nodeset(nodeset) => nodeset.iter().map(|node| node.string_value()).collect(),
                _ => Vec::new(),
            };


        Ok(Artist {
               mbid: reader.read_mbid(".//mb:artist/@id")?,
               name: reader.evaluate(".//mb:artist/mb:name/text()")?.string(),
               sort_name: reader.evaluate(".//mb:artist/mb:sort-name/text()")?.string(),
               aliases: aliases,
               artist_type: reader.evaluate(".//mb:artist/@type")?.string().parse::<ArtistType>()?,
               gender: gender,
               area: area,
               begin_date: reader
                   .evaluate(".//mb:artist/mb:life-span/mb:begin/text()")?
                   .string()
                   .parse::<Date>()
                   .ok(),
               end_date: reader
                   .evaluate(".//mb:artist/mb:life-span/mb:end/text()")?
                   .string()
                   .parse::<Date>()
                   .ok(),
               ipi_code: non_empty_string(reader.evaluate(".//mb:artist/mb:ipi/text()")?.string()),
               isni_code: non_empty_string(reader
                                               .evaluate(".//mb:artist/mb:isni-list/mb:isni/text()")?
                                               .string()),
           })
    }
}

impl Resource for Artist {
    fn get_url(mbid: &str) -> String {
        format!("https://musicbrainz.org/ws/2/artist/{}?inc=aliases", mbid).to_string()
    }
}

pub struct Event {}

pub struct Instrument {}

#[derive(Debug, Eq, PartialEq)]
pub enum LabelType {
    /// The main `LabelType` in the MusicBrainz database.
    /// That is a brand (and trademark) associated with the marketing of a release.
    Imprint,

    /// Production company producing entirely new releases.
    ProductionOriginal,
    /// Known bootleg production companies, not sanctioned by the rights owners of the released
    /// work.
    ProductionBootleg,
    /// Companies specialized in catalog reissues.
    ProductionReissue,

    /// Companies mainly distributing other labels production, often in a specfic region of the
    /// world.
    Distribution,
    /// Holdings, conglomerates or other financial entities that don't mainly produce records but
    /// manage a large set of recording labels owned by them.
    Holding,
    /// An organization which collects royalties on behalf of the artists.
    RightsSociety,
}

impl FromStr for LabelType {
    type Err = ReadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Imprint" => Ok(LabelType::Imprint),
            "Original Production" => Ok(LabelType::ProductionOriginal),
            "Bootleg Production" => Ok(LabelType::ProductionBootleg),
            "Reissue Production" => Ok(LabelType::ProductionReissue),
            "Distribution" => Ok(LabelType::Distribution),
            "Holding" => Ok(LabelType::Holding),
            "RightsSociety" => Ok(LabelType::RightsSociety),
            s => Err(ReadErrorKind::InvalidData(format!("Invalid `LabelType`: '{}'", s).to_string()).into()),
        }
    }
}

pub struct Series {}

pub struct Work {}

pub struct Url {}

// TODO: rating, tag, collection
// TODO: discid, isrc, iswc

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artist_read_xml1() {
        // url: https://musicbrainz.org/ws/2/artist/90e7c2f9-273b-4d6c-a662-ab2d73ea4b8e?inc=aliases
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><artist type-id="e431f5f6-b5d2-343d-8b36-72607fffb74b" id="90e7c2f9-273b-4d6c-a662-ab2d73ea4b8e" type="Group"><name>NECRONOMIDOL</name><sort-name>NECRONOMIDOL</sort-name><country>JP</country><area id="2db42837-c832-3c27-b4a3-08198f75693c"><name>Japan</name><sort-name>Japan</sort-name><iso-3166-1-code-list><iso-3166-1-code>JP</iso-3166-1-code></iso-3166-1-code-list></area><begin-area id="8dc97297-ac95-4d33-82bc-e07fab26fb5f"><name>Tokyo</name><sort-name>Tokyo</sort-name><iso-3166-2-code-list><iso-3166-2-code>JP-13</iso-3166-2-code></iso-3166-2-code-list></begin-area><life-span><begin>2014-03</begin></life-span></artist></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let result = Artist::from_xml(&reader).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("90e7c2f9-273b-4d6c-a662-ab2d73ea4b8e").unwrap());
        assert_eq!(result.name, "NECRONOMIDOL".to_string());
        assert_eq!(result.sort_name, "NECRONOMIDOL".to_string());
        assert_eq!(result.aliases, Vec::<String>::new());

        assert_eq!(result.begin_date,
                   Some(Date::Month {
                            year: 2014,
                            month: 3,
                        }));
        assert_eq!(result.end_date, None);

        let area = result.area.unwrap();
        assert_eq!(area.mbid,
                   Mbid::parse_str("2db42837-c832-3c27-b4a3-08198f75693c").unwrap());
        assert_eq!(area.name, "Japan".to_string());
        assert_eq!(area.sort_name, "Japan".to_string());
        assert_eq!(area.iso_3166, Some("JP".to_string()));

        assert_eq!(result.artist_type, ArtistType::Group);
        assert_eq!(result.gender, None);
        assert_eq!(result.ipi_code, None);
        assert_eq!(result.isni_code, None);
    }

    #[test]
    fn artist_read_xml2() {
        // url: https://musicbrainz.org/ws/2/artist/650e7db6-b795-4eb5-a702-5ea2fc46c848?inc=aliases
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><artist id="650e7db6-b795-4eb5-a702-5ea2fc46c848" type="Person" type-id="b6e035f4-3ce9-331c-97df-83397230b0df"><name>Lady Gaga</name><sort-name>Lady Gaga</sort-name><ipi>00519338442</ipi><ipi-list><ipi>00519338442</ipi><ipi>00519338540</ipi></ipi-list><isni-list><isni>0000000120254559</isni></isni-list><gender id="93452b5a-a947-30c8-934f-6a4056b151c2">Female</gender><country>US</country><area id="489ce91b-6658-3307-9877-795b68554c98"><name>United States</name><sort-name>United States</sort-name><iso-3166-1-code-list><iso-3166-1-code>US</iso-3166-1-code></iso-3166-1-code-list></area><begin-area id="261962ea-d8c2-4eaf-a80c-f14376ffadb0"><name>Manhattan</name><sort-name>Manhattan</sort-name></begin-area><life-span><begin>1986-03-28</begin></life-span><alias-list count="2"><alias sort-name="Lady Ga Ga">Lady Ga Ga</alias><alias type="Legal name" sort-name="Germanotta, Stefani Joanne Angelina" type-id="d4dcd0c0-b341-3612-a332-c0ce797b25cf">Stefani Joanne Angelina Germanotta</alias></alias-list></artist></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let result = Artist::from_xml(&reader).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("650e7db6-b795-4eb5-a702-5ea2fc46c848").unwrap());
        assert_eq!(result.name, "Lady Gaga".to_string());
        assert_eq!(result.sort_name, "Lady Gaga".to_string());
        let mut aliases_sorted = result.aliases.clone();
        aliases_sorted.sort();
        assert_eq!(aliases_sorted,
                   vec!["Lady Ga Ga".to_string(),
                        "Stefani Joanne Angelina Germanotta".to_string()]);

        assert_eq!(result.begin_date,
                   Some(Date::Day {
                            year: 1986,
                            month: 3,
                            day: 28,
                        }));
        assert_eq!(result.end_date, None);

        let area = result.area.unwrap();
        assert_eq!(area.mbid,
                   Mbid::parse_str("489ce91b-6658-3307-9877-795b68554c98").unwrap());
        assert_eq!(area.name, "United States".to_string());
        assert_eq!(area.sort_name, "United States".to_string());
        assert_eq!(area.iso_3166, Some("US".to_string()));

        assert_eq!(result.artist_type, ArtistType::Person);
        assert_eq!(result.gender, Some(Gender::Female));
        assert_eq!(result.ipi_code, Some("00519338442".to_string()));
        assert_eq!(result.isni_code, Some("0000000120254559".to_string()));
    }

}
