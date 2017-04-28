use super::*;

pub enum EventType {
    Concert,
    Festival,
    LaunchEvent,
    ConventionExpo,
    MasterclassClinic
}

/* TODO
impl FromStr for EventType {
    type Err = ReadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Concert" => Ok(
    }
}
*/

pub struct Event {
    /// MBID of the entity in the MusicBrainz database.
    mbid: Mbid,
    
    /// The official name of the event or a descriptive name if the event doesn't have an official
    /// name.
    name: String,

    /// Describes what type of event this is exactly.
    event_type: EventType,

    /// True if the event was cancelled.
    cancelled: bool,

    /// List of songs played at the event.
    ///
    /// This is provided in an extensive text format, for which parsing is not yet implemented.
    /// (TODO: If anyone needs this functionality.)
    setlist: String,

    begin_date: Date,
    end_date: Date,

// TODO:    start_time: Time

    aliases: Vec<String>,

    disambiguation: Option<String>,

    annotation: Option<String>
}

// TODO implement reader 
