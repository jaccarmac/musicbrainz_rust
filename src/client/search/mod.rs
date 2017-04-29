use super::*;
use super::super::entities as full_entities;
use super::super::entities::ReleaseGroup;
use hyper::Url;

pub mod fields;
use self::fields::SearchField;

pub mod entities;

pub trait SearchBuilder {
    type SearchEntity;
    type FullEntity : Resource;

    fn build_url(&self, base_url: &str) -> Result<Url, ClientError>;
}

/// One entry of the search results.
pub struct SearchEntry<E>
{
    /// The returned entity.
    entity: E,

    /// A value from 0 to 100 indicating in percent how much this specific search result matches
    /// the search query.
    score: u8
}

/*
macro_rules! define_search_builder {
    ( $builder:ident, $fields:ident, $entity:ident ) => {
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

            /// Add a new parameter to the query.
            pub fn add<F>(&mut self, field: &F) -> &mut Self
                where F: $fields
            {
                self.params.push((F::name(), field.to_string()));
                self
            }
        }

        impl<'cl> SearchBuilder for $builder<'cl> {
            type SearchEntity = entities::$entity;
            type FullEntity = full_entities::$entity;

            fn build_url(&self, base_url: &str) -> Result<Url, ClientError> {
                Ok(Url::parse_with_params(base_url, &self.params)?)
            }
/*
            fn search(&self) -> Result<Self::Entity, ClientError> {
                let url = self.build_url(Self::Entity::base_url())?;
                
            }
            */
        }
    }
}

define_search_builder!(ReleaseGroupSearchBuilder, ReleaseGroupSearchField, ReleaseGroup);

*/

