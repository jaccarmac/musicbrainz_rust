use super::*;
use hyper::Url;

pub trait SearchField {
    type Value: ToString;
    fn name() -> &'static str;
    fn to_string(&self) -> String;
}

/// For now only including the search fields of release group.
pub mod fields {
    use super::*;
    use super::super::super::entities;


    macro_rules! define_field {
        ( $type:ident, $name:expr, $value:ty ) => {
            pub struct $type ( pub $value );
            impl SearchField for $type {
                type Value = $value;

                fn name() -> &'static str {
                    $name
                }

                fn to_string(&self) -> String {
                    self.0.to_string()
                }
            }
        };
    }

    define_field!(ArtistId, "arid", Mbid);

    // TODO: release group : artist

    define_field!(ArtistName, "artistname", String);
    define_field!(Comment, "comment", String);
    define_field!(CreditName, "creditname", String);
    define_field!(PrimaryType, "primarytype", entities::ReleaseGroupPrimaryType);
    define_field!(ReleaseGroupId, "rgid", Mbid);
    define_field!(ReleaseGroupName, "releasegroup", String);
    define_field!(ReleaseGroupNameAccent, "releasegroupaccent", String);
    define_field!(ReleaseNumber, "releases", u16);
    define_field!(ReleaseName, "release", String);
    define_field!(ReleaseId, "reid", Mbid);
    define_field!(SecondaryType, "secondarytype", String);
    define_field!(ReleaseStatus, "status", entities::ReleaseStatus);
    define_field!(Tag, "tag", String);
}

macro_rules! register_search_fields {
    ( $entity:ident, $( $field:ident ),* ) => {
        pub trait $entity : SearchField { }

        $(
            impl $entity for fields::$field { }
        )*
    };
}

/// Acceptable fields when searching for a release group. TODO: Rethink where to put this docs.
register_search_fields!(ReleaseGroupSearchField, ArtistId, ArtistName, Comment, CreditName, PrimaryType, ReleaseGroupId, ReleaseGroupName, ReleaseGroupNameAccent, ReleaseNumber, ReleaseName, ReleaseId, SecondaryType, ReleaseStatus, Tag);

macro_rules! define_search_builder {
    ( $builder:ident, $fields:ident ) => {
        pub struct $builder {
            params: Vec<(&'static str, String)>
        }

        impl $builder {
            fn new() -> Self {
                Self {
                    params: Vec::new()
                }
            }

            fn build_url(&self, base_url: &str) -> Result<Url, ClientError> {
                Ok(Url::parse_with_params(base_url, &self.params)?)
            }

            /// Add a new parameter to the query.
            pub fn add<F>(&mut self, field: &F) -> &mut Self
                where F: $fields
            {
                self.params.push((F::name(), field.to_string()));
                self
            }
        }
    }
}

define_search_builder!(ReleaseGroupSearchBuilder, ReleaseGroupSearchField);
