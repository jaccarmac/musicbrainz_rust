use uuid;
use sxd_xpath::Value::Nodeset;
use std::str::FromStr;
/// TODO consider what type to use
pub use std::time::Duration;

mod xpath_reader;
use self::xpath_reader::*;
pub use self::xpath_reader::{FromXml, FromXmlContained, FromXmlElement, XPathStrReader};
use super::{ParseError, ParseErrorKind};

mod date;
pub use self::date::{Date, ParseDateError};

pub mod refs;
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

mod mbid;
pub use self::mbid::Mbid;

/// Takes a string and returns an option only containing the string if it was not empty.
fn non_empty_string(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

pub trait Resource {
    /// Returns the url where one can get a ressource in the valid format for parsing from.
    fn get_url(mbid: &Mbid) -> String;

    /// Base url of the entity, for example: `https://musicbrainz.org/ws/2/artist/`.
    /// These are used for searches for example.
    fn base_url() -> &'static str;
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
    type Err = ParseError;
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
                Err(ParseErrorKind::InvalidData(format!("Invalid `LabelType`: '{}'", s).to_string())
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


