/// ! For now only including the search fields of release group.

use super::{Mbid, full_entities};

pub trait SearchField {
    type Value: ToString;

    fn to_string(&self) -> String;
}

macro_rules! define_fields {
    ( $( $type:ident, $value:ty );* ) => {
        $(
            pub struct $type ( pub $value );

            impl SearchField for $type {
                type Value = $value;

                fn to_string(&self) -> String {
                    self.0.to_string()
                }
            }
        )*
    }
}

define_fields!(
    ArtistId, Mbid;
    ArtistCredit, String;
    ArtistName, String;
    Comment, String;
    CreditName, String;
    PrimaryType, full_entities::ReleaseGroupPrimaryType;
    ReleaseGroupId, Mbid;
    ReleaseGroupName, String;
    ReleaseGroupNameAccent, String;
    ReleaseNumber, u16;
    ReleaseName, String;
    ReleaseId, Mbid;
    SecondaryType, String;
    ReleaseStatus, full_entities::ReleaseStatus;
    Tag, String
);

macro_rules! define_entity_fields {
    (
        $field_trait:ident, $modname:ident;
        $(
            $field_type:ident, $strname:expr
        );*
    )
        =>
    {
        /// Acceptable fields searching for instances of the entity.
        pub trait $field_trait : SearchField {
            fn name() -> &'static str;
        }

        pub mod $modname {
            pub use super::$field_trait;

            $(
                pub use super::$field_type;

                impl $field_trait for $field_type {
                    fn name() -> &'static str { $strname }
                }
            )*
        }

    }
}

define_entity_fields!(
    ReleaseGroupSearchField, release_group;

    ArtistId, "arid";
    ArtistCredit, "artist";
    ArtistName, "artistname";
    Comment, "comment";
    CreditName, "creditname";
    PrimaryType, "primarytype";
    ReleaseGroupId, "rgid";
    ReleaseGroupName, "releasegroup";
    ReleaseGroupNameAccent, "releasegroupaccent";
    ReleaseNumber, "releases";
    ReleaseName, "release";
    ReleaseId, "reid";
    SecondaryType, "secondarytype";
    ReleaseStatus, "status";
    Tag, "tag"
);
