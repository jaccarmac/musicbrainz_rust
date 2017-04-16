use uuid;
use sxd_xpath::Value::Nodeset;
use std::str::FromStr;

mod xpath_reader;
use self::xpath_reader::*;
pub use self::xpath_reader::{ReadError, SxdParserError, SxdXpathError};

mod date;
pub use self::date::{Date, ParseDateError};

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
        let mbid = reader.read_mbid("//mb:area/@id")?;

        let area_type = reader
            .evaluate("//mb:area/@type")?
            .string()
            .parse::<AreaType>()?;
        let name = reader.evaluate("//mb:area/mb:name/text()")?.string();
        let sort_name = reader
            .evaluate("//mb:area/mb:sort-name/text()")?
            .string();
        let iso_3166_str = reader
            .evaluate("//mb:area/mb:iso-3166-1-code-list/mb:iso-3166-1-code/text()")?
            .string();

        Ok(Area {
               mbid: mbid,
               name: name,
               sort_name: sort_name,
               area_type: area_type,
               iso_3166: non_empty_string(iso_3166_str),
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

    /*
    /// The area an artist is primarily identified with. Often, but not always, birth/formation
    /// country of the artist/group.
    ///
    /// TODO: Consider if we should usa a different type than Area here, e.g. ArtistArea.
    /// The problem is that unlike with the standalone Area entity, the type is not provided here
    /// by the api and we have to make the area_type an Option in Area which is a bit ugly
    /// considering API users.
    /// (But we could also just leave it as is and let API clients deal with extracting all the
    /// optional values.)
    pub area: Option<Area>,
    */

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
        let sort_name = reader
            .evaluate("//mb:artist/mb:sort-name/text()")?
            .string();
        let artist_type = reader
            .evaluate("//mb:artist/@type")?
            .string()
            .parse::<ArtistType>()?;

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

        /*
        // Get area information.
        let area_val = match reader.evaluate("//mb:artist/mb:area")? {
            Nodeset(nodeset) => {
                if let Some(node) = nodeset.document_order_first() {
                    let reader = XPathNodeReader::new(node, &context)?;
                    Some(Area::read_xml(&reader)?)
                } else {
                    return Err(ReadError::InvalidData("Area element is empty.".to_string()));
                }
            }
            _ => None,
        };*/

        // Get IPI code.
        let ipi = non_empty_string(reader.evaluate("//mb:artist/mb:ipi/text()")?.string());

        // Get ISNI code.
        let isni = non_empty_string(reader
                                        .evaluate("//mb:artist/mb:isni-list/mb:isni/text()")?
                                        .string());

        Ok(Artist {
               mbid: mbid,
               name: name,
               sort_name: sort_name,
               artist_type: artist_type,
               gender: gender,
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

    pub date_begin: Option<Date>,
    pub date_end: Option<Date>,
}

impl FromXml for Label {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Label, ReadError>
        where R: XPathReader<'d>
    {
        let mbid = reader.read_mbid("//mb:label/@id")?;
        let name = reader.evaluate("//mb:label/mb:name/text()")?.string();
        let sort_name = reader
            .evaluate("//mb:label/mb:sort-name/text()")?
            .string();
        let disambiguation = non_empty_string(reader
                                                  .evaluate("//mb:label/mb:disambiguation/text()")?
                                                  .string());
        let aliases = Vec::new(); // TODO
        let label_code = non_empty_string(reader
                                              .evaluate("//mb:label/mb:label-code/text()")?
                                              .string());
        let label_type = reader
            .evaluate("//mb:label/@type")?
            .string()
            .parse::<LabelType>()?;
        let country = non_empty_string(reader
                                           .evaluate("//mb:label/mb:country/text()")?
                                           .string());
        let ipi_code = None; // TODO
        let isni_code = None; // TODO
        let date_begin = Date::from_str(&reader
                                             .evaluate("//mb:label/mb:life-span/mb:begin/text()")?
                                             .string()
                                             [..])
                .ok();
        let date_end = Date::from_str(&reader
                                           .evaluate("//mb:label/mb:life-span/mb:end/text()")?
                                           .string()
                                           [..])
                .ok();

        Ok(Label {
               mbid: mbid,
               name: name,
               sort_name: sort_name,
               disambiguation: disambiguation,
               aliases: aliases,
               label_code: label_code,
               label_type: label_type,
               country: country,
               ipi_code: ipi_code,
               isni_code: isni_code,
               date_begin: date_begin,
               date_end: date_end,
           })
    }
}

pub struct Recording {}

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

pub struct Release {
    /// MBID of the entity in the MusicBrainz database.
    pub mbid: Mbid,

    /// The title of the release.
    pub title: String,

    // TODO: `label`

    // TODO: The date the release was issued.
//    pub date: Date
    pub country: String,

    pub status: ReleaseStatus,

    // TODO: packaging
    /// Language of the release. ISO 639-3 conformant string.
    pub language: String,

    /// Script used to write the track list. ISO 15924 conformant string.
    pub script: String,

    // TODO: disamuiguation comments
    // TODO: annotations
    pub barcode: Option<String>,
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

        /*
        let area = result.area.unwrap();
        assert_eq!(area.mbid,
                   Mbid::parse_str("489ce91b-6658-3307-9877-795b68554c98").unwrap());
        assert_eq!(area.name, "United States".to_string());
        assert_eq!(area.sort_name, "United States".to_string());
        assert_eq!(area.iso_3166, Some("US".to_string()));
        */

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
}
