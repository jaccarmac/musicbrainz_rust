//! Search entities.
//!
//! Don't confuse these with the entities in the top level module `entities`.
//! They are only contained in search results and provide a means to retrive
//! the full entitity a further API request.

use super::{Client, ClientError, full_entities};
use self::full_entities::refs::*;
use self::full_entities::{Mbid, Resource};
use xpath_reader::FromXmlError;
use xpath_reader::reader::{FromXml, FromXmlElement, XpathReader};

pub trait SearchEntity {
    /// The full entity that is refered by this search entity.
    type FullEntity: Resource + FromXml;

    /// Fetch the full entity from the API.
    fn fetch_full(&self, client: &mut Client) -> Result<Self::FullEntity, ClientError>;
}

// It's the same entity.
pub use self::full_entities::Area;

impl SearchEntity for Area {
    type FullEntity = Area;

    fn fetch_full(&self, _: &mut Client) -> Result<Self::FullEntity, ClientError>
    {
        Ok(self.to_owned())
    }
}

pub use self::full_entities::Artist;

impl SearchEntity for Artist {
    type FullEntity = Artist;

    fn fetch_full(&self, _: &mut Client) -> Result<Self::FullEntity, ClientError>
    {
        Ok(self.to_owned())
    }
}

pub struct Release {
    pub mbid: Mbid,
    pub title: String,
    pub status: full_entities::ReleaseStatus,
    pub language: Option<String>,
    pub script: Option<String>,
    pub artists: Vec<ArtistRef>,
    // release group refs (TODO)
}

impl SearchEntity for Release {
    type FullEntity = full_entities::Release;

    fn fetch_full(&self, client: &mut Client) -> Result<Self::FullEntity, ClientError>
    {
        client.get_by_mbid(&self.mbid)
    }
}

pub struct ReleaseGroup {
    pub mbid: Mbid,
    pub title: String,
    pub artists: Vec<ArtistRef>,
    pub releases: Vec<ReleaseRef>,
}

impl SearchEntity for ReleaseGroup {
    type FullEntity = full_entities::ReleaseGroup;

    fn fetch_full(&self, client: &mut Client) -> Result<Self::FullEntity, ClientError>
    {
        client.get_by_mbid(&self.mbid)
    }
}

impl FromXmlElement for ReleaseGroup {}
impl FromXml for ReleaseGroup {
    fn from_xml<'d, R>(reader: &'d R) -> Result<Self, FromXmlError>
    where
        R: XpathReader<'d>,
    {
        Ok(ReleaseGroup {
            mbid: reader.read(".//@id")?,
            title: reader.read(".//mb:title")?,
            artists: reader.read_vec(".//mb:artist-credit/mb:name-credit/mb:artist")?,
            releases: reader.read_vec(".//mb:release-list/mb:release")?,
        })
    }
}
