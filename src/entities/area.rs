use xpath_reader::{FromXml, FromXmlError, XpathReader};
use xpath_reader::reader::{FromXmlContained, FromXmlElement};

use entities::{Mbid, Resource};

enum_mb_xml!{
    /// Specifies what a specific `Area` instance actually is.
    pub enum AreaType {
        /// Areas included (or previously included) in ISO 3166-1.
        var Country = "Country",

        /// Main administrative divisions of a countryr
        var Subdivision = "Subdivision",

        /// Smaller administrative divisions of a country, which are not one of the
        /// main administrative
        /// divisions but are also not muncipalities.
        var County = "County",

        /// Small administrative divisions. Urban municipalities often contain only
        /// a single city and a
        /// few surrounding villages, while rural municipalities often group several
        /// villages together.
        var Municipality = "Municipality",

        /// Settlements of any size, including towns and villages.
        var City = "City",

        /// Used for a division of a large city.
        var District = "District",

        /// Islands and atolls which don't form subdivisions of their own.
        var Island = "Island",
    }
}

/// A geographic region or settlement.
///
/// The exact type is distinguished by the `area_type` field.
/// This is one of the *core entities* of MusicBrainz.
///
/// [MusicBrainz documentation](https://musicbrainz.org/doc/Area).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Area {
    /// MBID of the entity in the MusicBrainz database.
    pub mbid: Mbid,

    /// The name of the area.
    pub name: String,

    /// Name that is supposed to be used for sorting, containing only latin
    /// characters.
    pub sort_name: String,

    /// Type of the area, gives more information about
    pub area_type: AreaType,

    /// ISO 3166 code, assigned to countries and subdivisions.
    pub iso_3166: Option<String>,
}

impl FromXmlContained for Area {}
impl FromXml for Area {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Area, FromXmlError>
    where
        R: XpathReader<'d>,
    {
        Ok(Area {
            mbid: reader.read(".//mb:area/@id")?,
            name: reader.read(".//mb:area/mb:name/text()")?,
            sort_name: reader.read(".//mb:area/mb:sort-name/text()")?,
            area_type: reader.read(".//mb:area/@type")?,
            iso_3166: reader.read_option(
                ".//mb:area/mb:iso-3166-1-code-list/mb:iso-3166-1-code/text()",
            )?,
        })
    }
}

impl Resource for Area {
    fn get_name() -> &'static str
    {
        "area"
    }

    fn get_incs() -> &'static str
    {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn area_read_xml1()
    {
        let mbid = Mbid::from_str("a1411661-be21-4290-8dc1-50f3d8e3ea67").unwrap();
        let area: Area = ::util::test_utils::fetch_entity(&mbid).unwrap();

        assert_eq!(area.mbid, mbid);
        assert_eq!(area.name, "Honolulu".to_string());
        assert_eq!(area.sort_name, "Honolulu".to_string());
        assert_eq!(area.area_type, AreaType::City);
        assert_eq!(area.iso_3166, None);
    }

    #[test]
    fn area_read_xml2()
    {
        let mbid = Mbid::from_str("2db42837-c832-3c27-b4a3-08198f75693c").unwrap();
        let area: Area = ::util::test_utils::fetch_entity(&mbid).unwrap();

        assert_eq!(area.mbid, mbid);
        assert_eq!(area.name, "Japan".to_string());
        assert_eq!(area.sort_name, "Japan".to_string());
        assert_eq!(area.area_type, AreaType::Country);
        assert_eq!(area.iso_3166, Some("JP".to_string()));
    }
}
