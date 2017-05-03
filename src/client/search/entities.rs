///! Search entities.
///! Don't confuse these with the entities in the top level module `entities`.
///! They are only contained in search results and provide a means to retrive the full entitity
///! a further API request.

use super::{full_entities, Client, ClientError, Mbid};
use self::full_entities::refs::*;
use self::full_entities::{Resource, FromXmlContained};

pub trait SearchEntity {
    /// The full entity that is refered by this search entity.
    type FullEntity: Resource + FromXmlContained;

    /// Fetch the full entity from the API.
    fn fetch_full(&self, client: &Client) -> Result<Self::FullEntity, ClientError>;
}

pub struct ReleaseGroup {
    pub mbid: Mbid,
    pub title: String,
    pub artists: Vec<ArtistRef>,
    pub releases: Vec<ReleaseRef>,
}

impl SearchEntity for ReleaseGroup {
    type FullEntity = full_entities::ReleaseGroup;

    fn fetch_full(&self, client: &Client) -> Result<Self::FullEntity, ClientError> {
        client.get_by_mbid(&self.mbid)
    }
}
