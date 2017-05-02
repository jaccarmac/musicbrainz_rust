use super::{hyper, ParseError, ClientError};
use super::entities::{Mbid, Resource};

use std::io::Read;

pub mod search;

/// Configuration for the client.
pub struct ClientConfig {
    /// The user-agent to be sent with every request to the API.
    ///
    /// Provide a meaningful one as it will be used by MusicBrainz to identify your application and
    /// without a user agent sever throttling will be undertaken. The official suggestion is to use
    /// either one of the following two options:
    ///
    /// * `Application name/<version> ( contact-url )`
    /// * `Application name/<version> ( contact-email )`
    ///
    /// For more information see: https://musicbrainz.org/doc/XML_Web_Service/Rate_Limiting
    pub user_agent: String,
}

pub struct Client {
    config: ClientConfig,
    http_client: hyper::Client,
}

impl Client {
    pub fn new(config: ClientConfig) -> Self {
        Client {
            config: config,
            http_client: hyper::Client::new(),
        }
    }

    /// Fetch the specified ressource from the server and parse it.
    pub fn get_by_mbid<Res>(&self, mbid: &Mbid) -> Result<Res, ClientError>
        where Res: Resource + FromXmlContained
    {
        use hyper::header::UserAgent;

        let url = Res::get_url(mbid);
        let mut response = self.http_client
            .get(&url[..])
            .header(UserAgent(self.config.user_agent.clone()))
            .send()?;
        let mut response_body = String::new();
        response.read_to_string(&mut response_body)?;

        // Parse the response.
        let reader = XpathStrReader::new(&response_body[..])?;
        Ok(Res::from_xml(&reader)?)
    }

    /*
    pub fn search_release_group<'cl>(&'cl self) -> ReleaseGroupSearchBuilder<'cl> {
        ReleaseGroupSearchBuilder::new(self)
    }
    */
}
