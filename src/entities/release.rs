use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseTrack {
    /// MBID of the entity in the MusicBrainz database.
    pub mbid: Mbid,

    // TODO: docstring
    pub position: u16,
    // TODO: docstring
    pub number: u16,

    /// The title of the track.
    pub title: String,

    /// The length of the track.
    pub length: Duration,

    /// The recording used for the track.
    pub recording: RecordingRef,
}

impl FromXml for ReleaseTrack {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        let mbid = reader.read_mbid(".//@id")?;
        Ok(ReleaseTrack {
               mbid: mbid,
               position: reader.evaluate(".//mb:position/text()")?.string().parse()?,
               number: reader.evaluate(".//mb:number/text()")?.string().parse()?,
               title: reader.evaluate(".//mb:title/text()")?.string(),
               length: Duration::from_millis(reader.evaluate(".//mb:length/text()")?
                                                 .string()
                                                 .parse()?),
               recording: {
                   match reader.evaluate(".//mb:recording")? {
                       Nodeset(nodeset) => {
                           if let Some(node) = nodeset.document_order_first() {
                               let context = default_musicbrainz_context();
                               let reader = XPathNodeReader::new(node, &context)?;
                               RecordingRef::from_xml(&reader)?
                           } else {
                               return Err(ReadErrorKind::InvalidData(format!("ReleaseTrack without RecordingRef, mbid: {}", mbid).to_string()).into());
                           }
                       }
                       _ => return Err(ReadErrorKind::InvalidData(format!("ReleaseTrack without RecordingRef, mbid: {}", mbid).to_string()).into()),
                   }
               },
           })
    }
}

/// A medium is a collection of multiple `ReleaseTrack`. For physical releases one medium might
/// equal one CD, so an album released as a release with two CDs would have two associated
/// `ReleaseMedium` instances.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseMedium {
    /// The medium's position number providing a total order between all mediums of one `Release`.
    position: u16,
    /// The tracks stored on this medium.
    tracks: Vec<ReleaseTrack>,
}

impl FromXml for ReleaseMedium {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        // TODO: test offset for multi cd releases.
        let tracks_node = reader.evaluate(".//mb:track-list/mb:track")?;
        let tracks = match tracks_node {
            Nodeset(nodeset) => {
                let context = default_musicbrainz_context();
                let res: Result<Vec<ReleaseTrack>, ReadError> = nodeset.document_order().iter().map(|node| {
                    XPathNodeReader::new(*node, &context).and_then(|r| ReleaseTrack::from_xml(&r))
                }).collect();
                res?
            }
            _ => Vec::new(),
        };
        Ok(ReleaseMedium {
               position: reader.evaluate(".//mb:position/text()")?.string().parse()?,
               tracks: tracks,
           })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
                Err(ReadErrorKind::InvalidData(format!("Unknown `ReleaseStatus`: '{}'", s)
                                                   .to_string())
                            .into())
            }
        }
    }
}
#[derive(Clone, Debug)]
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

    /// The mediums (disks) of the release.
    pub mediums: Vec<ReleaseMedium>,
}

impl FromXml for Release {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        let context = default_musicbrainz_context();
        let artists_node = reader.evaluate(".//mb:release/mb:artist-credit/mb:name-credit")?;
        let artists = match artists_node {
            Nodeset(nodeset) => {
                let res: Result<Vec<ArtistRef>, ReadError> = nodeset.iter().map(|node| {
                    XPathNodeReader::new(node, &context).and_then(|r| ArtistRef::from_xml(&r))
                }).collect();
                res?
            }
            _ => Vec::new(),
        };

        let labels_node = reader.evaluate(".//mb:release/mb:label-info-list/mb:label-info")?;
        let labels = match labels_node {
            Nodeset(nodeset) => {
                let res: Result<Vec<LabelRef>, ReadError> = nodeset.document_order().iter().map(|node| {
                    XPathNodeReader::new(*node, &context).and_then(|r| LabelRef::from_xml(&r))
                }).collect();
                res?
            }
            _ => Vec::new(),
        };

        let mediums_node = reader.evaluate(".//mb:release/mb:medium-list/mb:medium")?;
        let mediums = match mediums_node {
            Nodeset(nodeset) => {
                let res: Result<Vec<ReleaseMedium>, ReadError> = nodeset.document_order().iter().map(|node| {
                    XPathNodeReader::new(*node, &context).and_then(|r| ReleaseMedium::from_xml(&r))
                }).collect();
                res?
            }
            _ => Vec::new(),
        };

        Ok(Release {
               mbid: reader.read_mbid(".//mb:release/@id")?,
               title: reader.evaluate(".//mb:release/mb:title/text()")?.string(),
               artists: artists,
               date: reader.evaluate(".//mb:release/mb:date/text()")?.string().parse::<Date>()?,
               country: reader.evaluate(".//mb:release/mb:country/text()")?.string(),
               labels: labels,
               catalogue_number: non_empty_string(
                   reader.evaluate(".//mb:release/mb:label-info-list/mb:label-info/mb:catalog-number/text()")?.string()),
               barcode: non_empty_string(reader
                                             .evaluate(".//mb:release/mb:barcode/text()")?
                                             .string()),
               status: reader
                   .evaluate(".//mb:release/mb:status/text()")?
                   .string()
                   .parse::<ReleaseStatus>()?,
               packaging: non_empty_string(reader.evaluate(".//mb:release/mb:packaging/text()")?.string()),
               language: reader
                   .evaluate(".//mb:release/mb:text-representation/mb:language/text()")?
                   .string(),
               script: reader
                   .evaluate(".//mb:release/mb:text-representation/mb:script/text()")?
                   .string(),
               disambiguation:
                   non_empty_string(reader
                                        .evaluate(".//mb:release/mb:disambiguation/text()")?
                                        .string()),
               mediums: mediums
           })
    }
}

impl Resource for Release {
    fn get_url(mbid: &str) -> String {
        format!("https://musicbrainz.org/ws/2/release/{}?inc=aliases+artists+labels+recordings",
                mbid)
                .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(release.catalogue_number, Some("CDR 6078".to_string()));
        assert_eq!(release.barcode, Some("724388023429".to_string()));
        assert_eq!(release.status, ReleaseStatus::Official);
        assert_eq!(release.language, "eng".to_string());
        assert_eq!(release.script, "Latn".to_string());
        // TODO: check disambiguation
        //assert_eq!(release.disambiguation,
        assert_eq!(release.mediums, Vec::new());
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
        assert_eq!(release.mediums, Vec::new());
    }

    #[test]
    fn read_tracks() {
        // url: https://musicbrainz.org/ws/2/release/d1881a4c-0188-4f0f-a2e7-4e7849aec109?inc=artists+labels+recordings
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><release id="d1881a4c-0188-4f0f-a2e7-4e7849aec109"><title>EXITIUM</title><status id="4e304316-386d-3409-af2e-78857eec5cfe">Official</status><quality>normal</quality><text-representation><language>jpn</language><script>Jpan</script></text-representation><artist-credit><name-credit><artist id="90e7c2f9-273b-4d6c-a662-ab2d73ea4b8e"><name>NECRONOMIDOL</name><sort-name>NECRONOMIDOL</sort-name></artist></name-credit></artist-credit><date>2015-10-04</date><country>JP</country><release-event-list count="1"><release-event><date>2015-10-04</date><area id="2db42837-c832-3c27-b4a3-08198f75693c"><name>Japan</name><sort-name>Japan</sort-name><iso-3166-1-code-list><iso-3166-1-code>JP</iso-3166-1-code></iso-3166-1-code-list></area></release-event></release-event-list><asin>B014GUVIM8</asin><cover-art-archive><artwork>false</artwork><count>0</count><front>false</front><back>false</back></cover-art-archive><label-info-list count="1"><label-info><label id="58592b07-de7e-4231-9b0b-4b9c9e1f3a03"><name>VELOCITRON</name><sort-name>VELOCITRON</sort-name></label></label-info></label-info-list><medium-list count="1"><medium><position>1</position><track-list offset="0" count="3"><track id="ac898be7-2965-4d17-9ac8-48d45852d73c"><position>1</position><number>1</number><title>puella tenebrarum</title><length>232000</length><recording id="fd6f4cd8-9cff-43da-8cd7-3351357b6f5a"><title>Puella Tenebrarum</title><length>232000</length></recording></track><track id="21648b0b-deaf-4b93-a257-5fc18363b25d"><position>2</position><number>2</number><title>LAMINA MALEDICTUM</title><length>258000</length><recording id="0eeb0621-8013-4c0e-8e49-ddfd78d56051"><title>Lamina Maledictum</title><length>258000</length></recording></track><track id="e57b3990-eb36-476e-beac-583e0bbe6f87"><position>3</position><number>3</number><title>SARNATH</title><length>228000</length><recording id="53f87e98-351e-453e-b949-bdacf4cbeccd"><title>Sarnath</title><length>228000</length></recording></track></track-list></medium></medium-list></release></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let release = Release::from_xml(&reader).unwrap();

        let mediums = release.mediums;
        assert_eq!(mediums.len(), 1);
        let medium = mediums.get(0).unwrap();
        assert_eq!(medium.position, 1);
        assert_eq!(medium.tracks.len(), 3);
        assert_eq!(medium.tracks[0],
                   ReleaseTrack {
                       mbid: Mbid::parse_str("ac898be7-2965-4d17-9ac8-48d45852d73c").unwrap(),
                       position: 1,
                       number: 1,
                       title: "puella tenebrarum".to_string(),
                       length: Duration::from_millis(232000),
                       recording: RecordingRef {
                           mbid: Mbid::parse_str("fd6f4cd8-9cff-43da-8cd7-3351357b6f5a").unwrap(),
                           title: "Puella Tenebrarum".to_string(),
                           length: Duration::from_millis(232000),
                       },
                   });
        assert_eq!(medium.tracks[1],
                   ReleaseTrack {
                       mbid: Mbid::parse_str("21648b0b-deaf-4b93-a257-5fc18363b25d").unwrap(),
                       position: 2,
                       number: 2,
                       title: "LAMINA MALEDICTUM".to_string(),
                       length: Duration::from_millis(258000),
                       recording: RecordingRef {
                           mbid: Mbid::parse_str("0eeb0621-8013-4c0e-8e49-ddfd78d56051").unwrap(),
                           title: "Lamina Maledictum".to_string(),
                           length: Duration::from_millis(258000),
                       },
                   });
        assert_eq!(medium.tracks[2],
                   ReleaseTrack {
                       mbid: Mbid::parse_str("e57b3990-eb36-476e-beac-583e0bbe6f87").unwrap(),
                       position: 3,
                       number: 3,
                       title: "SARNATH".to_string(),
                       length: Duration::from_millis(228000),
                       recording: RecordingRef {
                           mbid: Mbid::parse_str("53f87e98-351e-453e-b949-bdacf4cbeccd").unwrap(),
                           title: "Sarnath".to_string(),
                           length: Duration::from_millis(228000),
                       },
                   });
    }
}
