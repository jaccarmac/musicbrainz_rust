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

impl FromXmlContained for AreaRef {}
impl FromXml for AreaRef {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        Ok(AreaRef {
               mbid: reader.read_mbid(".//mb:area/@id")?,
               name: reader.read_string(".//mb:area/mb:name/text()")?,
               sort_name: reader.read_string(".//mb:area/mb:sort-name/text()")?,
               iso_3166: reader.read_nstring(".//mb:area/mb:iso-3166-1-code-list/mb:iso-3166-1-code/text()")?,
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

impl FromXmlElement for ArtistRef {}
impl FromXml for ArtistRef {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        Ok(ArtistRef {
               mbid: reader.read_mbid(".//@id")?,
               name: reader.read_string(".//mb:name/text()")?,
               sort_name: reader.read_string(".//mb:sort-name/text()")?,
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

impl FromXmlElement for LabelRef {}
impl FromXml for LabelRef {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        Ok(LabelRef {
               mbid: reader.read_mbid(".//@id")?,
               name: reader.read_string(".//mb:name/text()")?,
               sort_name: reader.read_string(".//mb:sort-name/text()")?,
               label_code: reader.read_nstring(".//mb:label-code/text()")?
           })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingRef {
    pub mbid: Mbid,
    pub title: String,
    pub length: Duration
}

impl FromXmlElement for RecordingRef {}
impl FromXml for RecordingRef {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        Ok(RecordingRef {
            mbid: reader.read_mbid(".//@id")?,
            title: reader.read_string(".//mb:title/text()")?,
            // TODO reader.read<Duration>
            length: Duration::from_millis(reader.evaluate(".//mb:length/text()")?.string().parse::<u64>()?)
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseRef {
    pub mbid: Mbid,
    pub title: String,
    pub date: Date,
    pub status: ReleaseStatus,
    pub country: String
}

impl FromXmlElement for ReleaseRef {}
impl FromXml for ReleaseRef {
    /// reader root at : `release` element which is the `ReleaseRef` to be parsed.
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, ReadError>
        where R: XPathReader<'d>
    {
        Ok(ReleaseRef {
            mbid: reader.read_mbid(".//@id")?,
            title: reader.read_string(".//mb:title/text()")?,
            date: reader.read_date(".//mb:date/text()")?,
            status: reader.read_string(".//mb:status/text()")?.parse()?,
            country: reader.read_string(".//mb:country/text()")?
        })
    }
}

