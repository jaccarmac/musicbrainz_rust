use super::*;
use super::super::entities as full_entities;
use hyper::Url;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

use xpath_reader::{FromXml, XpathError, XpathReader};

pub mod fields;
use self::fields::ReleaseGroupSearchField;

pub mod entities;
use self::entities::SearchEntity;

pub type SearchResult<Entity> = Result<Vec<SearchEntry<Entity>>, ClientError>;

pub trait SearchBuilder {
    /// The entity from the client::search::entities module,
    /// this is the entity contained in the search result.
    type Entity: entities::SearchEntity;

    /// The full entity a search entity can be expanded into.
    type FullEntity: Resource + FromXml;

    /// Perform the search.
    fn search(self) -> SearchResult<Self::Entity>;
}

/// One entry of the search results.
pub struct SearchEntry<E>
    where E: SearchEntity
{
    /// The returned entity.
    pub entity: E,

    /// A value from 0 to 100 indicating in percent how much this specific
    /// search result matches
    /// the search query.
    pub score: u8,
}

macro_rules! define_search_builder {
    ( $builder:ident,
      $fields:ident,
      $entity:ty,
      $full_entity:ty,
      $list_tag:expr ) => {
        pub struct $builder<'cl> {
            params: Vec<(&'static str, String)>,
            client: &'cl super::Client,
        }

        impl<'cl> $builder<'cl> {
            pub fn new(client: &'cl super::Client) -> Self {
                Self {
                    params: Vec::new(),
                    client: client,
                }
            }

            /// Specify an additional parameter for the query.
            pub fn add<F>(mut self, field: F) -> Self
                where F: $fields
            {
                self.params.push((F::name(), field.to_string()));
                self
            }

            // TODO: In the future support OR queries too.
            fn build_url(&self) -> Result<Url, ClientError> {
                let mut query_parts: Vec<String> = Vec::new();
                for &(p_name, ref p_value) in self.params.iter() {
                    let value  = utf8_percent_encode(p_value.as_ref(), DEFAULT_ENCODE_SET);
                    query_parts.push(format!("{}:{}", p_name, value));
                }

                let query = query_parts.join("%20AND%20");
                type FE = $full_entity;
                Ok(Url::parse(format!("{}?query={}", FE::base_url(), query).as_ref())?)
            }
        }

        impl<'cl> SearchBuilder for $builder<'cl> {
            type Entity = $entity;
            type FullEntity = $full_entity;

            fn search(self) -> SearchResult<Self::Entity> {
                use entities::default_musicbrainz_context;

//                let url = Url::parse_with_params(Self::FullEntity::base_url(), &self.params)?;
                let url = self.build_url()?;
                println!("search url: {}", url);

                // Perform the request.
                let response_body = self.client.get_body(url)?;
                let context = default_musicbrainz_context();
                let reader = XpathStrReader::new(response_body.as_str(), &context)?;

                Ok(reader.read_vec("//mb:metadata")?)
            }
        }

        impl FromXml for SearchEntry<$entity> {
            fn from_xml<'d, R>(reader: &'d R) -> Result<Self, XpathError>
                where R: XpathReader<'d>
            {
                Ok(Self {
                    entity: reader.read(format!(".//mb:{}", $list_tag).as_str())?,
                    score: reader.read(format!(".//mb:{}/@count", $list_tag).as_str())?,
                })
            }
        }
    }
}

define_search_builder!(ReleaseGroupSearchBuilder,
                       ReleaseGroupSearchField,
                       entities::ReleaseGroup,
                       full_entities::ReleaseGroup,
                       "release-group-list");
