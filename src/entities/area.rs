use super::*;

/// Specifies what a specific `Area` instance actually is.
#[derive(Debug, Clone, Eq, PartialEq)]
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
            s => Err(ReadErrorKind::InvalidData(format!("Invalid `AreaType`: '{}'", s)).into()),
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
               mbid: reader.read_mbid(".//mb:area/@id")?,
               name: reader.read_string(".//mb:area/mb:name/text()")?,
               sort_name: reader.read_string(".//mb:area/mb:sort-name/text()")?,
               area_type: reader.read_string(".//mb:area/@type")?.parse()?,
               iso_3166: reader.read_nstring(".//mb:area/mb:iso-3166-1-code-list/mb:iso-3166-1-code/text()")?,
           })
    }
}

impl Resource for Area {
    fn get_url(mbid: &str) -> String {
        format!("https://musicbrainz.org/ws/2/area/{}", mbid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_read_xml1() {
        // url: https://musicbrainz.org/ws/2/area/a1411661-be21-4290-8dc1-50f3d8e3ea67
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><area type="City" type-id="6fd8f29a-3d0a-32fc-980d-ea697b69da78" id="a1411661-be21-4290-8dc1-50f3d8e3ea67"><name>Honolulu</name><sort-name>Honolulu</sort-name></area></metadata>"#;

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
        // url: https://musicbrainz.org/ws/2/area/2db42837-c832-3c27-b4a3-08198f75693c
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><area type="Country" id="2db42837-c832-3c27-b4a3-08198f75693c" type-id="06dd0ae4-8c74-30bb-b43d-95dcedf961de"><name>Japan</name><sort-name>Japan</sort-name><iso-3166-1-code-list><iso-3166-1-code>JP</iso-3166-1-code></iso-3166-1-code-list></area></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let result = Area::from_xml(&reader).unwrap();

        assert_eq!(result.mbid,
                   Mbid::parse_str("2db42837-c832-3c27-b4a3-08198f75693c").unwrap());
        assert_eq!(result.name, "Japan".to_string());
        assert_eq!(result.sort_name, "Japan".to_string());
        assert_eq!(result.area_type, AreaType::Country);
        assert_eq!(result.iso_3166, Some("JP".to_string()));
    }
}
