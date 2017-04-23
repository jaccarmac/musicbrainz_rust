//! This module contains structs for types we call *reference types* in this library.
//!
//! These types only contain some basic data but reference a full entity in the MusicBrainz
//! database which can be retrieved.

// TODO: Better documentation in this file.
// TODO: When writing the API interfacing code, provide some form of helpers so the full referenced
//       types corresponding to these ref types can be easily retrieved from the server.

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AreaRef {
    pub mbid: Mbid,
    pub name: String,
    pub sort_name: String,
    pub iso_3166: Option<String>,
}

impl FromXml for AreaRef {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        Ok(AreaRef {
               mbid: reader.read_mbid(".//mb:area/@id")?,
               name: reader.evaluate(".//mb:area/mb:name/text()")?.string(),
               sort_name: reader.evaluate(".//mb:area/mb:sort-name/text()")?.string(),
               iso_3166: non_empty_string(reader.evaluate(".//mb:area/mb:iso-3166-1-code-list/mb:iso-3166-1-code/text()")?.string()),
           })
    }
}

/// A small variation of `Artist` which is used only to refer to an actual artist entity from other
/// entities.
/// TODO: new docstring
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtistRef {
    pub mbid: Mbid,
    pub name: String,
    pub sort_name: String,
}

impl FromXml for ArtistRef {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        Ok(ArtistRef {
               mbid: reader.read_mbid(".//mb:artist/@id")?,
               name: reader.evaluate(".//mb:artist/mb:name/text()")?.string(),
               sort_name: reader.evaluate(".//mb:artist/mb:sort-name/text()")?.string(),
           })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabelRef {
    pub mbid: Mbid,
    pub name: String,
    pub sort_name: String,
    pub label_code: Option<String>,
}

impl FromXml for LabelRef {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        Ok(LabelRef {
               mbid: reader.read_mbid(".//mb:label/@id")?,
               name: reader.evaluate(".//mb:label/mb:name/text()")?.string(),
               sort_name: reader.evaluate(".//mb:label/mb:sort-name/text()")?.string(),
               label_code: non_empty_string(reader
                                                .evaluate(".//mb:label/mb:label-code/text()")?
                                                .string()),
           })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingRef {
    pub mbid: Mbid,
    pub title: String,
    pub length: Duration
}

impl FromXml for RecordingRef {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        Ok(RecordingRef {
            mbid: reader.read_mbid(".//@id")?,
            title: reader.evaluate(".//mb:title/text()")?.string(),
            length: Duration::from_millis(reader.evaluate(".//mb:length/text()")?.string().parse::<u64>()?)
        })
    }
}

