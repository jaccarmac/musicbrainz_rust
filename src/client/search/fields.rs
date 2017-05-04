/// ! For now only including the search fields of release group.

use super::{Mbid, full_entities};

pub trait SearchField {
    type Value: ToString;
    fn name() -> &'static str;
    fn to_string(&self) -> String;
}

// TODO: Wrap fields for each entity into a submodule so there are no clashes
// and users can quickly
// lookup relevant fields using autocomplete.
// In my attempts so far Iwasn't able to achieve that due to module scoping
// issues.
macro_rules! define_fields {
    (
        $field_trait:ident, $modname:ident;
        $(
            $type:ident, $name:expr, $value:ty
        );*
    ) => {
        /// Acceptable fields searching for instances of the entity.
        pub trait $field_trait : SearchField { }

        $(
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

            impl $field_trait for $type { }
        )*
    };
}

// TODO: release group : artist
define_fields!(
    ReleaseGroupSearchField, release_group;

    ArtistId, "arid", Mbid;
    ArtistName, "artistname", String;
    Comment, "comment", String;
    CreditName, "creditname", String;
    PrimaryType, "primarytype", full_entities::ReleaseGroupPrimaryType;
    ReleaseGroupId, "rgid", Mbid;
    ReleaseGroupName, "releasegroup", String;
    ReleaseGroupNameAccent, "releasegroupaccent", String;
    ReleaseNumber, "releases", u16;
    ReleaseName, "release", String;
    ReleaseId, "reid", Mbid;
    SecondaryType, "secondarytype", String;
    ReleaseStatus, "status", full_entities::ReleaseStatus;
    Tag, "tag", String
);
