use super::*;

/// The primary type of a release group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReleaseGroupPrimaryType {
    Album,
    Single,
    EP,
    Broadcast,
    Other,
}

impl FromStr for ReleaseGroupPrimaryType {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Album" => Ok(ReleaseGroupPrimaryType::Album),
            "Single" => Ok(ReleaseGroupPrimaryType::Single),
            "EP" => Ok(ReleaseGroupPrimaryType::EP),
            "Broadcast" => Ok(ReleaseGroupPrimaryType::Broadcast),
            "Other" => Ok(ReleaseGroupPrimaryType::Other),
            _ => {
                Err(ParseErrorKind::InvalidData(format!("Unknown ReleaseGroupPrimaryType: '{}'", s)
                                                   .to_string())
                            .into())
            }
        }
    }
}

/// Secondary types of a release group. There can be any number of secondary types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReleaseGroupSecondaryType {
    Compilation,
    Soundtrack,
    Spokenword,
    Interview,
    Audiobook,
    Live,
    Remix,
    DjMix,
    MixtapeStreet,
}

impl FromStr for ReleaseGroupSecondaryType {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Compilation" => Ok(ReleaseGroupSecondaryType::Compilation),
            "Soundtrack" => Ok(ReleaseGroupSecondaryType::Soundtrack),
            "Spokenword" => Ok(ReleaseGroupSecondaryType::Spokenword),
            "Interview" => Ok(ReleaseGroupSecondaryType::Interview),
            "Audiobook" => Ok(ReleaseGroupSecondaryType::Audiobook),
            "Live" => Ok(ReleaseGroupSecondaryType::Live),
            "Remix" => Ok(ReleaseGroupSecondaryType::Remix),
            "DJ-mix" => Ok(ReleaseGroupSecondaryType::DjMix),
            "Mixtape/Street" => Ok(ReleaseGroupSecondaryType::MixtapeStreet),
            _ => {
                Err(ParseErrorKind::InvalidData(format!("Unknown ReleaseSecondaryPrimaryType: '{}'",
                                                       s)
                                                       .to_string())
                            .into())
            }
        }
    }
}

/// The type of a `ReleaseGroup`.
///
/// For more information consult: https://musicbrainz.org/doc/Release_Group/Type
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseGroupType {
    pub primary: Option<ReleaseGroupPrimaryType>,
    pub secondary: Vec<ReleaseGroupSecondaryType>,
}

impl FromXmlElement for ReleaseGroupType {}
impl FromXml for ReleaseGroupType {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ParseError>
        where R: XPathReader<'d>
    {
        use sxd_xpath::Value::Nodeset;

        Ok(ReleaseGroupType {
               primary: match reader.read_nstring(".//mb:primary-type/text()")? {
                   Some(s) => Some(s.parse()?),
                   None => None,
               },
               secondary:
                   match reader.evaluate(".//mb:secondary-type-list/mb:secondary-type/text()")? {
                       Nodeset(nodeset) => {
                let r: Result<Vec<ReleaseGroupSecondaryType>, ParseError> = nodeset.iter().map(|node| {
                            node.text()
                                .ok_or_else(|| ParseErrorKind::InvalidData("ReleaseGroupType read xml failure: invalid node structure.".to_string()).into())
                                .and_then(|s| s.text().parse())
                            }).collect();
                r?
            }
                       _ => Vec::new(),
                   },
           })
    }
}

/// Groups multiple `Release`s into one a single logical entity.
/// Even if there is only one release of a kind, it belongs to exactly one release group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseGroup {
    /// MBID of the entity in the MusicBrainz database.
    pub mbid: Mbid,

    /// Title of the release group, usually the same as the title of the releases.
    pub title: String,

    /// The artists of a release group.
    pub artists: Vec<ArtistRef>,

    /// Releases of this releaes group.
    pub releases: Vec<ReleaseRef>,

    /// The type of this release group.
    pub release_type: ReleaseGroupType,

    // TODO docstring
    pub disambiguation: Option<String>,

    // TODO: docstring
    pub annotation: Option<String>,
}

impl Resource for ReleaseGroup {
    fn get_url(mbid: &Mbid) -> String {
        format!("https://musicbrainz.org/ws/2/release-group/{}?inc=annotation+artists+releases",
                mbid.hyphenated())
    }
}

impl FromXmlContained for ReleaseGroup {}
impl FromXml for ReleaseGroup {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ParseError>
        where R: XPathReader<'d>
    {
        Ok(ReleaseGroup {
               mbid: reader.read_mbid(".//mb:release-group/@id")?,
               title: reader.read_string(".//mb:release-group/mb:title/text()")?,
               releases: reader.read_vec(".//mb:release-group/mb:release-list/mb:release")?,
               artists:
                   reader.read_vec(".//mb:release-group/mb:artist-credit/mb:name-credit/mb:artist")?,
               release_type: {
                   let rel_reader = reader.relative_reader(".//mb:release-group")?;
                   ReleaseGroupType::from_xml(&rel_reader)?
               },
               disambiguation: reader.read_nstring(".//mb:release-group/mb:disambiguation/text()")?,
               annotation: reader.read_nstring(".//mb:release-group/mb:annotation/text()")?,
           })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_1() {
        // url: https://musicbrainz.org/ws/2/release-group/76a4e2c2-bf7a-445e-8081-5a1e291f3b16?inc=annotation+artists+releases
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><release-group type="Album" id="76a4e2c2-bf7a-445e-8081-5a1e291f3b16" type-id="f529b476-6e62-324f-b0aa-1f3e33d313fc"><title>Mixtape</title><first-release-date>2012-03</first-release-date><primary-type id="f529b476-6e62-324f-b0aa-1f3e33d313fc">Album</primary-type><secondary-type-list><secondary-type id="15c1b1f5-d893-3375-a1db-e180c5ae15ed">Mixtape/Street</secondary-type></secondary-type-list><artist-credit><name-credit><artist id="0e6b3a2c-6a42-4b43-a4f6-c6625c5855de"><name>POP ETC</name><sort-name>POP ETC</sort-name></artist></name-credit></artist-credit><release-list count="1"><release id="289bf4e7-0af5-433c-b5a2-493b863b4b47"><title>Mixtape</title><status id="4e304316-386d-3409-af2e-78857eec5cfe">Official</status><quality>normal</quality><text-representation><language>eng</language><script>Latn</script></text-representation><date>2012-03</date><country>US</country><release-event-list count="1"><release-event><date>2012-03</date><area id="489ce91b-6658-3307-9877-795b68554c98"><name>United States</name><sort-name>United States</sort-name><iso-3166-1-code-list><iso-3166-1-code>US</iso-3166-1-code></iso-3166-1-code-list></area></release-event></release-event-list></release></release-list></release-group></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let rg = ReleaseGroup::from_xml(&reader).unwrap();

        assert_eq!(rg.mbid,
                   Mbid::parse_str("76a4e2c2-bf7a-445e-8081-5a1e291f3b16").unwrap());
        assert_eq!(rg.title, "Mixtape".to_string());
        assert_eq!(rg.artists,
                   vec![ArtistRef {
                            mbid: Mbid::parse_str("0e6b3a2c-6a42-4b43-a4f6-c6625c5855de").unwrap(),
                            name: "POP ETC".to_string(),
                            sort_name: "POP ETC".to_string(),
                        }]);
        assert_eq!(rg.releases,
                   vec![ReleaseRef {
                            mbid: Mbid::parse_str("289bf4e7-0af5-433c-b5a2-493b863b4b47").unwrap(),
                            title: "Mixtape".to_string(),
                            date: Date::Month {
                                year: 2012,
                                month: 03,
                            },
                            status: ReleaseStatus::Official,
                            country: "US".to_string(),
                        }]);
        assert_eq!(rg.release_type.primary,
                   Some(ReleaseGroupPrimaryType::Album));
        assert_eq!(rg.release_type.secondary,
                   vec![ReleaseGroupSecondaryType::MixtapeStreet]);
        assert_eq!(rg.disambiguation, None);
        assert_eq!(rg.annotation, None);
    }
}
