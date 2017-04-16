use uuid;
use sxd_xpath::Value::Nodeset;
use std::str::FromStr;
/// TODO consider what type to use
pub use std::time::Duration;

mod xpath_reader;
use self::xpath_reader::*;
pub use self::xpath_reader::{ReadError, SxdParserError, SxdXpathError};

mod date;
pub use self::date::{Date, ParseDateError};

mod refs;
pub use self::refs::{AreaRef, ArtistRef, LabelRef};

/// Identifier for entities in the MusicBrainz database.
/// TODO: Figure out if it makes more sense to keep
pub type Mbid = uuid::Uuid;

/// Takes a string and returns an option only containing the string if it was not empty.
fn non_empty_string(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

pub trait FromXml
    where Self: Sized
{
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError> where R: XPathReader<'d>;
}

pub trait Resource {
    /// Returns the url where one can get a ressource in the valid format for parsing from.
    fn get_url(mbid: &str) -> String;
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

impl FromStr for AreaType {
    type Err = ReadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Country" => Ok(AreaType::Country),
            "Subdivision" => Ok(AreaType::Subdivision),
            "County" => Ok(AreaType::County),
            "Muncipality" => Ok(AreaType::Muncipality),
            "City" => Ok(AreaType::City),
            "District" => Ok(AreaType::District),
            "Island" => Ok(AreaType::Island),
            s => Err(ReadError::InvalidData(format!("Invalid `AreaType`: '{}'", s).to_string())),
        }
    }
}

/// A geographic region or settlement.
/// The exact type is distinguished by the `area_type` field.
/// This is one of the *core entities* of MusicBrainz.
///
/// https://musicbrainz.org/doc/Area
pub struct Area {
    /// MBID of the entity in the MusicBrainz database.
    pub mbid: Mbid,

    /// The name of the area.
    pub name: String,

    /// Name that is supposed to be used for sorting, containing only latin characters.
    pub sort_name: String,

    /// The type of the area.
    pub area_type: AreaType,

    /// ISO 3166 code, assigned to countries and subdivisions.
    pub iso_3166: Option<String>,
}


impl FromXml for Area {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Area, ReadError>
        where R: XPathReader<'d>
    {
        Ok(Area {
               mbid: reader.read_mbid("//mb:area/@id")?,
               name: reader.evaluate("//mb:area/mb:name/text()")?.string(),
               sort_name: reader.evaluate("//mb:area/mb:sort-name/text()")?.string(),
               area_type: reader.evaluate("//mb:area/@type")?.string().parse::<AreaType>()?,
               iso_3166: non_empty_string(reader.evaluate("//mb:area/mb:iso-3166-1-code-list/mb:iso-3166-1-code/text()")?.string()),
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
                return Err(ReadError::InvalidData(format!("Unknown artist type: {}", t)
                                                      .to_string()))
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

    /// Whether this Artist is a person, group, or something else.
    pub artist_type: ArtistType,

    /// If the Artist is a single person this indicates their gender.
    pub gender: Option<Gender>,

    /// The area an artist is primarily identified with. Often, but not always, birth/formation
    /// country of the artist/group.
    pub area: Option<AreaRef>,

    // TODO: begin and end dates
    pub ipi_code: Option<String>,
    pub isni_code: Option<String>, /* TODO aliases
                                    * TODO disambiguation comment */
}

impl FromXml for Artist {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        let mbid = reader.read_mbid("//mb:artist/@id")?;
        let name = reader.evaluate("//mb:artist/mb:name/text()")?.string();
        let sort_name = reader.evaluate("//mb:artist/mb:sort-name/text()")?.string();
        let artist_type = reader.evaluate("//mb:artist/@type")?.string().parse::<ArtistType>()?;

        // Get gender.
        let gender = match reader.evaluate("//mb:artist/mb:gender/text()") {
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

        let area = match reader.evaluate("//mb:artist") {
            Ok(Nodeset(nodeset)) => {
                if let Some(node) = nodeset.document_order_first() {
                    let context = default_musicbrainz_context();
                    let reader = XPathNodeReader::new(node, &context)?;
                    Some(AreaRef::from_xml(&reader)?)
                } else {
                    None
                }
            }
            _ => None
        };

        // Get IPI code.
        let ipi = non_empty_string(reader.evaluate("//mb:artist/mb:ipi/text()")?.string());

        // Get ISNI code.
        let isni =
            non_empty_string(reader.evaluate("//mb:artist/mb:isni-list/mb:isni/text()")?.string());

        Ok(Artist {
               mbid: mbid,
               name: name,
               sort_name: sort_name,
               artist_type: artist_type,
               gender: gender,
               area: area,
               ipi_code: ipi,
               isni_code: isni,
           })
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
            s => Err(ReadError::InvalidData(format!("Invalid `LabelType`: '{}'", s).to_string())),
        }
    }
}

/// A label entity in the MusicBrainz database.
/// There is quite some controversy in the music industry what a 'label' constitutes.
///
/// For a complete disambiguation see the `LabelType` enum. The labels in MusicBrainz are mostly
/// imprints.
pub struct Label {
    /// MBID of the entity in the MusicBrainz database.
    pub mbid: Mbid,

    /// The official name of the label.
    pub name: String,

    /// Version of the `name` converted to latin characters for sorting.
    pub sort_name: String,

    /// If there are multiple labels with the same name in the database, a short disambiguation
    /// comment is provided which allows to differentiate the entities.
    pub disambiguation: Option<String>,

    /// Variants of the name mainly used as search help.
    /// These can be variants, spellings of names, missing titles and common misspellings.
    pub aliases: Vec<String>,

    /// LC code of the label, as issued by the IFPI.
    pub label_code: Option<String>,

    /// Describes the main activity of the label.
    pub label_type: LabelType,

    /// ISO 3166 country of origin for the label.
    pub country: Option<String>,

    /// Identifying number of the label as assigned by the CISAC database.
    pub ipi_code: Option<String>,

    /// ISNI code of the label.
    pub isni_code: Option<String>,

    /// TODO: docs
    pub date_begin: Option<Date>,

    /// TODO: docs
    pub date_end: Option<Date>,
}

impl FromXml for Label {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Label, ReadError>
        where R: XPathReader<'d>
    {
        Ok(Label {
               mbid: reader.read_mbid("//mb:label/@id")?,
               name: reader.evaluate("//mb:label/mb:name/text()")?.string(),
               sort_name: reader.evaluate("//mb:label/mb:sort-name/text()")?.string(),
               disambiguation:
                   non_empty_string(reader
                                        .evaluate("//mb:label/mb:disambiguation/text()")?
                                        .string()),
               aliases: Vec::new(), // TODO
               label_code: non_empty_string(reader
                                                .evaluate("//mb:label/mb:label-code/text()")?
                                                .string()),
               label_type: reader.evaluate("//mb:label/@type")?.string().parse::<LabelType>()?,
               country: non_empty_string(reader.evaluate("//mb:label/mb:country/text()")?.string()),
               ipi_code: None, // TODO
               isni_code: None, // TODO
               date_begin: reader
                   .evaluate("//mb:label/mb:life-span/mb:begin/text()")?
                   .string()
                   .parse::<Date>()
                   .ok(),
               date_end: reader
                   .evaluate("//mb:label/mb:life-span/mb:end/text()")?
                   .string()
                   .parse::<Date>()
                   .ok(),
           })
    }
}

/// Represents a unique audio that has been used to produce at least one released track through
/// copying or mastering.
#[derive(Clone, Debug)]
pub struct Recording {
    /// MBID of the entity in the MusicBrainz database.
    pub mbid: Mbid,

    /// The title of the recording.
    pub title: String,

    /// The artists that the recording is primarily credited to.
    pub artists: Vec<ArtistRef>,

    /// Approximation of the length of the recording, calculated from the tracks using it.
    pub duration: Duration,

    /// ISRC (International Standard Recording Code) assigned to the recording.
    pub isrc_code: Option<String>,

    /// Disambiguation comment.
    pub disambiguation: Option<String>,

    /// Annotation if present.
    pub annotation: Option<String>,
}

impl FromXml for Recording {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        let artists = Vec::new();
        Ok(Recording {
               mbid: reader.read_mbid("//mb:recording/@id")?,
               title: reader.evaluate("//mb:recording/mb:title/text()")?.string(),
               artists: artists,
               duration: Duration::from_millis(reader
                                                   .evaluate("//mb:recording/mb:length/text()")?
                                                   .string()
                                                   .parse::<u64>()?),
               isrc_code: None, // TODO,
               disambiguation:
                   non_empty_string(reader
                                        .evaluate("//mb:recording/mb:disambiguation/text()")?
                                        .string()),
               annotation: non_empty_string(reader
                                                .evaluate("//mb:recording/mb:annotation/text()")?
                                                .string()),
           })
    }
}

impl Resource for Recording {
    fn get_url(mbid: &str) -> String {
        format!("https://musicbrainz.org/ws/2/recording/{}?inc=artists+",
                mbid)
                .to_string()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ReleaseStatus {
    /// Release officially sanctioned by the artist and/or their record company.
    Official,
    /// A give-away release or a release intended to promote an upcoming official release.
    Promotional,
    /// Unofficial/underground release that was not sanctioned by the artist and/or the record
    /// company. Includes unoffcial live recordings and pirated releases.
    Bootleg,
    /// An alternate version of a release where the titles have been changed.
    /// These don't correspond to any real release and should be linked to the original release
    /// using the transliteration relationship.
    ///
    /// TL;DR: Essentially this shouldn't be used.
    PseudoRelease,
}

impl FromStr for ReleaseStatus {
    type Err = ReadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Official" => Ok(ReleaseStatus::Official),
            "Promotional" => Ok(ReleaseStatus::Promotional),
            "Bootleg" => Ok(ReleaseStatus::Bootleg),
            "PseudoRelease" => Ok(ReleaseStatus::PseudoRelease),
            s => {
                Err(ReadError::InvalidData(format!("Unknown `ReleaseStatus`: '{}'", s).to_string()))
            }
        }
    }
}

pub struct Release {
    /// MBID of the entity in the MusicBrainz database.
    pub mbid: Mbid,

    /// The title of the release.
    pub title: String,

    /// The artists that the release is primarily credited to.
    pub artists: Vec<ArtistRef>,

    /// The date the release was issued.
    pub date: Date,

    /// The country the release was issued in.
    pub country: String,

    /// The label which issued this release.
    pub labels: Vec<LabelRef>,

    /// Number assigned to the release by the label.
    pub catalogue_number: Option<String>,

    /// Barcode of the release, if it has one.
    pub barcode: Option<String>,

    /// Official status of the release.
    pub status: ReleaseStatus,

    /// Packaging of the release.
    /// TODO: Consider an enum for the possible packaging types.
    pub packaging: Option<String>,

    /// Language of the release. ISO 639-3 conformant string.
    pub language: String,

    /// Script used to write the track list. ISO 15924 conformant string.
    pub script: String,

    /// A disambiguation comment if present, which allows to differentiate this release easily from
    /// other releases with the same or very similar name.
    pub disambiguation: Option<String>, // TODO: annotations
}

impl FromXml for Release {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        let context = default_musicbrainz_context();
        let artists_node = reader.evaluate("//mb:release/mb:artist-credit/mb:name-credit")?;
        let artists = match artists_node {
            Nodeset(nodeset) => {
                let res: Result<Vec<ArtistRef>, ReadError> = nodeset.iter().map(|node| {
                    XPathNodeReader::new(node, &context).and_then(|r| ArtistRef::from_xml(&r))
                }).collect();
                res?
            }
            _ => Vec::new(),
        };

        let labels_node = reader.evaluate("//mb:release/mb:label-info-list/mb:label-info")?;
        let labels = match labels_node {
            Nodeset(nodeset) => {
                let res: Result<Vec<LabelRef>, ReadError> = nodeset.document_order().iter().map(|node| {
                    XPathNodeReader::new(*node, &context).and_then(|r| LabelRef::from_xml(&r))
                }).collect();
                res?
            }
            _ => Vec::new(),
        };

        Ok(Release {
               mbid: reader.read_mbid("//mb:release/@id")?,
               title: reader.evaluate("//mb:release/mb:title/text()")?.string(),
               artists: artists,
               date: reader.evaluate("//mb:release/mb:date/text()")?.string().parse::<Date>()?,
               country: reader.evaluate("//mb:release/mb:country/text()")?.string(),
               labels: labels,
               catalogue_number: non_empty_string(
                   reader.evaluate("//mb:release/mb:label-info-list/mb:label-info/mb:catalog-number/text()")?.string()),
               barcode: non_empty_string(reader
                                             .evaluate("//mb:release/mb:barcode/text()")?
                                             .string()),
               status: reader
                   .evaluate("//mb:release/mb:status/text()")?
                   .string()
                   .parse::<ReleaseStatus>()?,
               packaging: non_empty_string(reader.evaluate("//mb:release/mb:packaging/text()")?.string()),
               language: reader
                   .evaluate("//mb:release/mb:text-representation/mb:language/text()")?
                   .string(),
               script: reader
                   .evaluate("//mb:release/mb:text-representation/mb:script/text()")?
                   .string(),
               disambiguation:
                   non_empty_string(reader
                                        .evaluate("//mb:release/mb:disambiguation/text()")?
                                        .string()),
           })
    }
}

impl Resource for Release {
    fn get_url(mbid: &str) -> String {
        format!("https://musicbrainz.org/ws/2/release/{}?inc=aliases+artists+labels",
                mbid)
                .to_string()
    }
}

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
        let result = Area::from_xml(&reader).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("a1411661-be21-4290-8dc1-50f3d8e3ea67").unwrap());
        assert_eq!(result.name, "Honolulu".to_string());
        assert_eq!(result.sort_name, "Honolulu".to_string());
        assert_eq!(result.area_type, AreaType::City);
        assert_eq!(result.iso_3166, None);
    }

    #[test]
    fn area_read_xml2() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><area type-id="06dd0ae4-8c74-30bb-b43d-95dcedf961de" type="Country" id="2db42837-c832-3c27-b4a3-08198f75693c"><name>Japan</name><sort-name>Japan</sort-name><iso-3166-1-code-list><iso-3166-1-code>JP</iso-3166-1-code></iso-3166-1-code-list></area></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let result = Area::from_xml(&reader).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("2db42837-c832-3c27-b4a3-08198f75693c").unwrap());
        assert_eq!(result.name, "Japan".to_string());
        assert_eq!(result.sort_name, "Japan".to_string());
        assert_eq!(result.area_type, AreaType::Country);
        assert_eq!(result.iso_3166, Some("JP".to_string()));
    }

    #[test]
    fn artist_read_xml1() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><artist id="90e7c2f9-273b-4d6c-a662-ab2d73ea4b8e" type-id="e431f5f6-b5d2-343d-8b36-72607fffb74b" type="Group"><name>NECRONOMIDOL</name><sort-name>NECRONOMIDOL</sort-name><country>JP</country><area id="2db42837-c832-3c27-b4a3-08198f75693c"><name>Japan</name><sort-name>Japan</sort-name><iso-3166-1-code-list><iso-3166-1-code>JP</iso-3166-1-code></iso-3166-1-code-list></area><begin-area id="8dc97297-ac95-4d33-82bc-e07fab26fb5f"><name>Tokyo</name><sort-name>Tokyo</sort-name><iso-3166-2-code-list><iso-3166-2-code>JP-13</iso-3166-2-code></iso-3166-2-code-list></begin-area><life-span><begin>2014-03</begin></life-span></artist></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let result = Artist::from_xml(&reader).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("90e7c2f9-273b-4d6c-a662-ab2d73ea4b8e").unwrap());
        assert_eq!(result.name, "NECRONOMIDOL".to_string());
        assert_eq!(result.sort_name, "NECRONOMIDOL".to_string());

        /*
        let area = result.area.unwrap();
        assert_eq!(area.mbid,
                   Mbid::parse_str("2db42837-c832-3c27-b4a3-08198f75693c").unwrap());
        assert_eq!(area.name, "Japan".to_string());
        assert_eq!(area.sort_name, "Japan".to_string());
        assert_eq!(area.iso_3166, Some("JP".to_string()));
        */

        assert_eq!(result.artist_type, ArtistType::Group);
        assert_eq!(result.gender, None);
        assert_eq!(result.ipi_code, None);
        assert_eq!(result.isni_code, None);
    }

    #[test]
    fn artist_read_xml2() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><artist type="Person" id="650e7db6-b795-4eb5-a702-5ea2fc46c848" type-id="b6e035f4-3ce9-331c-97df-83397230b0df"><name>Lady Gaga</name><sort-name>Lady Gaga</sort-name><ipi>00519338442</ipi><ipi-list><ipi>00519338442</ipi><ipi>00519338540</ipi></ipi-list><isni-list><isni>0000000120254559</isni></isni-list><gender id="93452b5a-a947-30c8-934f-6a4056b151c2">Female</gender><country>US</country><area id="489ce91b-6658-3307-9877-795b68554c98"><name>United States</name><sort-name>United States</sort-name><iso-3166-1-code-list><iso-3166-1-code>US</iso-3166-1-code></iso-3166-1-code-list></area><begin-area id="261962ea-d8c2-4eaf-a80c-f14376ffadb0"><name>Manhattan</name><sort-name>Manhattan</sort-name></begin-area><life-span><begin>1986-03-28</begin></life-span></artist></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let result = Artist::from_xml(&reader).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("650e7db6-b795-4eb5-a702-5ea2fc46c848").unwrap());
        assert_eq!(result.name, "Lady Gaga".to_string());
        assert_eq!(result.sort_name, "Lady Gaga".to_string());

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

    #[test]
    fn label_read_xml1() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><label id="c029628b-6633-439e-bcee-ed02e8a338f7" type="Original Production" type-id="7aaa37fe-2def-3476-b359-80245850062d"><name>EMI</name><sort-name>EMI</sort-name><disambiguation>EMI Records, since 1972</disambiguation><label-code>542</label-code><country>GB</country><area id="8a754a16-0027-3a29-b6d7-2b40ea0481ed"><name>United Kingdom</name><sort-name>United Kingdom</sort-name><iso-3166-1-code-list><iso-3166-1-code>GB</iso-3166-1-code></iso-3166-1-code-list></area><life-span><begin>1972</begin></life-span></label></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let label = Label::from_xml(&reader).unwrap();

        assert_eq!(label.mbid,
                   Mbid::parse_str("c029628b-6633-439e-bcee-ed02e8a338f7").unwrap());
        assert_eq!(label.name, "EMI".to_string());
        assert_eq!(label.sort_name, "EMI".to_string());
        assert_eq!(label.disambiguation,
                   Some("EMI Records, since 1972".to_string()));
        assert_eq!(label.aliases, Vec::<String>::new());
        assert_eq!(label.label_code, Some("542".to_string()));
        assert_eq!(label.label_type, LabelType::ProductionOriginal);
        assert_eq!(label.country, Some("GB".to_string()));
        assert_eq!(label.ipi_code, None);
        assert_eq!(label.isni_code, None);
        assert_eq!(label.date_begin, Some(Date::Year { year: 1972 }));
        assert_eq!(label.date_end, None);
    }

    #[test]
    fn release_read_xml1() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><release id="ed118c5f-d940-4b52-a37b-b1a205374abe"><title>Creep</title><status id="4e304316-386d-3409-af2e-78857eec5cfe">Official</status><quality>normal</quality><text-representation><language>eng</language><script>Latn</script></text-representation><artist-credit><name-credit><artist id="a74b1b7f-71a5-4011-9441-d0b5e4122711"><name>Radiohead</name><sort-name>Radiohead</sort-name></artist></name-credit></artist-credit><date>1992-09-21</date><country>GB</country><release-event-list count="1"><release-event><date>1992-09-21</date><area id="8a754a16-0027-3a29-b6d7-2b40ea0481ed"><name>United Kingdom</name><sort-name>United Kingdom</sort-name><iso-3166-1-code-list><iso-3166-1-code>GB</iso-3166-1-code></iso-3166-1-code-list></area></release-event></release-event-list><barcode>724388023429</barcode><asin>B000EHLKNU</asin><cover-art-archive><artwork>true</artwork><count>3</count><front>true</front><back>true</back></cover-art-archive><label-info-list count="1"><label-info><catalog-number>CDR 6078</catalog-number><label id="df7d1c7f-ef95-425f-8eef-445b3d7bcbd9"><name>Parlophone</name><sort-name>Parlophone</sort-name><label-code>299</label-code></label></label-info></label-info-list></release></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let release = Release::from_xml(&reader).unwrap();

        assert_eq!(release.mbid,
                   Mbid::parse_str("ed118c5f-d940-4b52-a37b-b1a205374abe").unwrap());
        assert_eq!(release.title, "Creep".to_string());
        assert_eq!(release.artists,
                   vec![ArtistRef {
                            mbid: Mbid::parse_str("a74b1b7f-71a5-4011-9441-d0b5e4122711").unwrap(),
                            name: "Radiohead".to_string(),
                            sort_name: "Radiohead".to_string(),
                        }]);
        assert_eq!(release.date, Date::from_str("1992-09-21").unwrap());
        assert_eq!(release.country, "GB".to_string());
        assert_eq!(release.labels,
                   vec![LabelRef {
                            mbid: Mbid::parse_str("df7d1c7f-ef95-425f-8eef-445b3d7bcbd9").unwrap(),
                            name: "Parlophone".to_string(),
                            sort_name: "Parlophone".to_string(),
                            label_code: Some("299".to_string()),
                        }]);
        // TODO: check labels.
        assert_eq!(release.catalogue_number, Some("CDR 6078".to_string()));
        assert_eq!(release.barcode, Some("724388023429".to_string()));
        assert_eq!(release.status, ReleaseStatus::Official);
        assert_eq!(release.language, "eng".to_string());
        assert_eq!(release.script, "Latn".to_string());
        // TODO: check disambiguation
        //assert_eq!(release.disambiguation,
    }

    #[test]
    fn release_read_xml2() {
        // url: https://musicbrainz.org/ws/2/release/785d7c67-a920-4cee-a871-8cd9896eb8aa?inc=aliases+artists+labels
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><release id="785d7c67-a920-4cee-a871-8cd9896eb8aa"><title>The Fame</title><status id="4e304316-386d-3409-af2e-78857eec5cfe">Official</status><quality>normal</quality><packaging id="ec27701a-4a22-37f4-bfac-6616e0f9750a">Jewel Case</packaging><text-representation><language>eng</language><script>Latn</script></text-representation><artist-credit><name-credit><artist id="650e7db6-b795-4eb5-a702-5ea2fc46c848"><name>Lady Gaga</name><sort-name>Lady Gaga</sort-name><alias-list count="2"><alias sort-name="Lady Ga Ga">Lady Ga Ga</alias><alias sort-name="Germanotta, Stefani Joanne Angelina" type-id="d4dcd0c0-b341-3612-a332-c0ce797b25cf" type="Legal name">Stefani Joanne Angelina Germanotta</alias></alias-list></artist></name-credit></artist-credit><date>2008-08-19</date><country>CA</country><release-event-list count="1"><release-event><date>2008-08-19</date><area id="71bbafaa-e825-3e15-8ca9-017dcad1748b"><name>Canada</name><sort-name>Canada</sort-name><iso-3166-1-code-list><iso-3166-1-code>CA</iso-3166-1-code></iso-3166-1-code-list></area></release-event></release-event-list><barcode>602517664890</barcode><asin>B001D25N2Y</asin><cover-art-archive><artwork>true</artwork><count>1</count><front>true</front><back>false</back></cover-art-archive><label-info-list count="5"><label-info><catalog-number>0251766489</catalog-number><label id="376d9b4d-8cdd-44be-bc0f-ed5dfd2d2340"><name>Cherrytree Records</name><sort-name>Cherrytree Records</sort-name></label></label-info><label-info><catalog-number>0251766489</catalog-number><label id="2182a316-c4bd-4605-936a-5e2fac52bdd2"><name>Interscope Records</name><sort-name>Interscope Records</sort-name><label-code>6406</label-code><alias-list count="3"><alias sort-name="Flip/Interscope Records">Flip/Interscope Records</alias><alias sort-name="Interscape Records">Interscape Records</alias><alias sort-name="Nothing/Interscope">Nothing/Interscope</alias></alias-list></label></label-info><label-info><catalog-number>0251766489</catalog-number><label id="061587cb-0262-46bc-9427-cb5e177c36a2"><name>Konlive</name><sort-name>Konlive</sort-name><alias-list count="1"><alias sort-name="Kon Live">Kon Live</alias></alias-list></label></label-info><label-info><catalog-number>0251766489</catalog-number><label id="244dd29f-b999-40e4-8238-cb760ad05ac6"><name>Streamline Records</name><sort-name>Streamline Records</sort-name><disambiguation>Interscope imprint</disambiguation></label></label-info><label-info><catalog-number>0251766489</catalog-number><label id="6cee07d5-4cc3-4555-a629-480590e0bebd"><name>Universal Music Canada</name><sort-name>Universal Music Canada</sort-name><disambiguation>1995–</disambiguation><alias-list count="2"><alias sort-name="Universal Music (Canada)">Universal Music (Canada)</alias><alias sort-name="Universal Music Canada in.">Universal Music Canada in.</alias></alias-list></label></label-info></label-info-list></release></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let release = Release::from_xml(&reader).unwrap();

        // We check for the things we didn't check in the previous test.
        assert_eq!(release.packaging, Some("Jewel Case".to_string()));
        assert_eq!(release.catalogue_number, Some("0251766489".to_string()));
        assert_eq!(release.labels,
                   vec![LabelRef {
                            mbid: Mbid::parse_str("376d9b4d-8cdd-44be-bc0f-ed5dfd2d2340").unwrap(),
                            name: "Cherrytree Records".to_string(),
                            sort_name: "Cherrytree Records".to_string(),
                            label_code: None,
                        },
                        LabelRef {
                            mbid: Mbid::parse_str("2182a316-c4bd-4605-936a-5e2fac52bdd2").unwrap(),
                            name: "Interscope Records".to_string(),
                            sort_name: "Interscope Records".to_string(),
                            label_code: Some("6406".to_string()),
                        },
                        LabelRef {
                            mbid: Mbid::parse_str("061587cb-0262-46bc-9427-cb5e177c36a2").unwrap(),
                            name: "Konlive".to_string(),
                            sort_name: "Konlive".to_string(),
                            label_code: None,
                        },
                        LabelRef {
                            mbid: Mbid::parse_str("244dd29f-b999-40e4-8238-cb760ad05ac6").unwrap(),
                            name: "Streamline Records".to_string(),
                            sort_name: "Streamline Records".to_string(),
                            label_code: None,
                        },
                        LabelRef {
                            mbid: Mbid::parse_str("6cee07d5-4cc3-4555-a629-480590e0bebd").unwrap(),
                            name: "Universal Music Canada".to_string(),
                            sort_name: "Universal Music Canada".to_string(),
                            label_code: None,
                        }]);
    }
}
