use uuid;
use sxd_xpath::Value::Nodeset;
use std::str::FromStr;
/// TODO consider what type to use
pub use std::time::Duration;

mod xpath_reader;
use self::xpath_reader::*;
pub use self::xpath_reader::{FromXml, FromXmlContained, FromXmlElement};
use super::{ReadError, ReadErrorKind};

mod date;
pub use self::date::{Date, ParseDateError};

mod refs;
pub use self::refs::{AreaRef, ArtistRef, LabelRef, RecordingRef, ReleaseRef};

mod area;
mod artist;
mod event;
mod label;
mod recording;
mod release;
mod release_group;
pub use self::area::{Area, AreaType};
pub use self::artist::{Artist, ArtistType, Gender};
pub use self::event::{Event, EventType};
pub use self::label::Label;
pub use self::recording::Recording;
pub use self::release::{Release, ReleaseTrack, ReleaseStatus, ReleaseMedium};
pub use self::release_group::{ReleaseGroup, ReleaseGroupType, ReleaseGroupPrimaryType,
                              ReleaseGroupSecondaryType};

/// Identifier for entities in the MusicBrainz database.
pub type Mbid = uuid::Uuid;

/// Takes a string and returns an option only containing the string if it was not empty.
fn non_empty_string(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}


pub trait Resource {
    /// Returns the url where one can get a ressource in the valid format for parsing from.
    fn get_url(mbid: &str) -> String;
}

pub struct Instrument {}

#[derive(Debug, Eq, PartialEq)]
pub enum LabelType {
    /// The main `LabelType` in the MusicBrainz database.
    /// That is a brand (and trademark) associated with the marketing of a release.
    Imprint,

    /// Production company producing entirely new releases.
    ProductionOriginal,
    /// Known bootleg production companies, not sanctioned by the rights owners of the released
    /// work.
    ProductionBootleg,
    /// Companies specialized in catalog reissues.
    ProductionReissue,

    /// Companies mainly distributing other labels production, often in a specfic region of the
    /// world.
    Distribution,
    /// Holdings, conglomerates or other financial entities that don't mainly produce records but
    /// manage a large set of recording labels owned by them.
    Holding,
    /// An organization which collects royalties on behalf of the artists.
    RightsSociety,
}

impl FromStr for LabelType {
    type Err = ReadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Imprint" => Ok(LabelType::Imprint),
            "Original Production" => Ok(LabelType::ProductionOriginal),
            "Bootleg Production" => Ok(LabelType::ProductionBootleg),
            "Reissue Production" => Ok(LabelType::ProductionReissue),
            "Distribution" => Ok(LabelType::Distribution),
            "Holding" => Ok(LabelType::Holding),
            "RightsSociety" => Ok(LabelType::RightsSociety),
            s => {
                Err(ReadErrorKind::InvalidData(format!("Invalid `LabelType`: '{}'", s).to_string())
                        .into())
            }
        }
    }
}

pub struct Series {}

pub struct Work {}

pub struct Url {}

// TODO: rating, tag, collection
// TODO: discid, isrc, iswc


