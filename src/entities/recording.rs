use super::*;

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
        Ok(Recording {
               mbid: reader.read_mbid(".//mb:recording/@id")?,
               title: reader.evaluate(".//mb:recording/mb:title/text()")?.string(),
               artists: match reader.evaluate(".//mb:recording/mb:artist-credit/mb:name-credit")? {
                   Nodeset(nodeset) => {
                       let context = default_musicbrainz_context();
                       let res: Result<Vec<ArtistRef>, ReadError> = nodeset
                           .iter()
                           .map(|node| {
                                    XPathNodeReader::new(node, &context)
                                        .and_then(|r| ArtistRef::from_xml(&r))
                                })
                           .collect();
                       res?
                   }
                   _ => Vec::new(),
               },
               duration: Duration::from_millis(reader
                                                   .evaluate(".//mb:recording/mb:length/text()")?
                                                   .string()
                                                   .parse::<u64>()?),
               isrc_code:
                   non_empty_string(reader
                                        .evaluate(".//mb:recording/mb:isrc-list/mb:isrc/@id")?
                                        .string()),
               disambiguation:
                   non_empty_string(reader
                                        .evaluate(".//mb:recording/mb:disambiguation/text()")?
                                        .string()),
               annotation: non_empty_string(reader
                                                .evaluate(".//mb:recording/mb:annotation/text()")?
                                                .string()),
           })
    }
}

impl Resource for Recording {
    fn get_url(mbid: &str) -> String {
        format!("https://musicbrainz.org/ws/2/recording/{}?inc=artists+annotation+isrcs",
                mbid)
                .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_xml1() {
        // url: https://musicbrainz.org/ws/2/recording/fbe3d0b9-3990-4a76-bddb-12f4a0447a2c?inc=artists+annotation+isrcs
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><metadata xmlns="http://musicbrainz.org/ns/mmd-2.0#"><recording id="fbe3d0b9-3990-4a76-bddb-12f4a0447a2c"><title>The Perfect Drug (Nine Inch Nails)</title><length>499000</length><artist-credit><name-credit><artist id="b7ffd2af-418f-4be2-bdd1-22f8b48613da"><name>Nine Inch Nails</name><sort-name>Nine Inch Nails</sort-name></artist></name-credit></artist-credit><isrc-list count="1"><isrc id="USIR19701296" /></isrc-list></recording></metadata>"#;
        let reader = XPathStrReader::new(xml).unwrap();
        let recording = Recording::from_xml(&reader).unwrap();

        assert_eq!(recording.mbid,
                   Mbid::parse_str("fbe3d0b9-3990-4a76-bddb-12f4a0447a2c").unwrap());
        assert_eq!(recording.title,
                   "The Perfect Drug (Nine Inch Nails)".to_string());
        assert_eq!(recording.duration, Duration::from_millis(499000));
        assert_eq!(recording.artists,
                   vec![ArtistRef {
                            mbid: Mbid::parse_str("b7ffd2af-418f-4be2-bdd1-22f8b48613da").unwrap(),
                            name: "Nine Inch Nails".to_string(),
                            sort_name: "Nine Inch Nails".to_string(),
                        }]);
        assert_eq!(recording.isrc_code, Some("USIR19701296".to_string()));
        assert_eq!(recording.annotation, None);
        assert_eq!(recording.disambiguation, None);
    }
}
