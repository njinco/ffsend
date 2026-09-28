use std::time::Duration;
#[cfg(feature = "send3")]
use std::{error::Error, io, net::TcpStream};

use reqwest::{
    self,
    blocking::{
        Client as ReqwestClient, ClientBuilder as ReqwestClientBuilder, Request as ReqwestRequest,
        RequestBuilder as ReqwestRequestBuilder, Response as ReqwestResponse,
    },
    IntoUrl,
};
#[cfg(feature = "send3")]
use tungstenite::{
    client::{connect_with_config, IntoClientRequest},
    http::{header, HeaderValue},
    stream::MaybeTlsStream,
    WebSocket,
};
#[cfg(feature = "send3")]
use url::Url;

/// The protocol to report to the server when uploading through a websocket.
#[cfg(feature = "send3")]
const WEBSOCKET_PROTOCOL: &str = "ffsend";

/// A networking client for ffsend actions.
pub struct Client {
    /// The client configuration to use for built reqwest clients.
    config: ClientConfig,

    /// The inner reqwest client.
    reqwest: ReqwestClient,
}

impl Client {
    /// Construct the ffsend client with the given `config`.
    ///
    /// If this client is used for transfering files, `transfer` should be true.
    // TODO: properly handle errors, do not unwrap
    pub fn new(config: ClientConfig, transfer: bool) -> Self {
        // Set up the reqwest client
        let mut builder = ReqwestClientBuilder::new();
        match config.timeout {
            Some(timeout) if !transfer => builder = builder.timeout(timeout),
            _ => {}
        }
        match config.transfer_timeout {
            Some(timeout) if transfer => builder = builder.timeout(timeout),
            _ => {}
        }
        let reqwest = builder.build().expect("failed to build reqwest client");

        // Build the client
        Self { config, reqwest }
    }

    /// Create a HTTP GET request through this client, returning a `RequestBuilder`.
    pub fn get<U: IntoUrl>(&self, url: U) -> ReqwestRequestBuilder {
        self.configure(self.reqwest.get(url))
    }

    /// Create a HTTP GET request through this client, returning a `RequestBuilder`.
    pub fn post<U: IntoUrl>(&self, url: U) -> ReqwestRequestBuilder {
        self.configure(self.reqwest.post(url))
    }

    /// Execute the given reqwest request through the internal reqwest client.
    // TODO: remove this, as the timeout can't be configured?
    pub fn execute(&self, request: ReqwestRequest) -> reqwest::Result<ReqwestResponse> {
        self.reqwest.execute(request)
    }

    /// Construct a websocket client connected to the given `url` using the wrapped configuration.
    #[cfg(feature = "send3")]
    pub fn websocket(
        &self,
        url: &str,
    ) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {
        // Send hosts use HTTP URLs, while the WebSocket handshake needs ws/wss.
        let mut ws_url = Url::parse(url)?;
        let scheme = match ws_url.scheme() {
            "http" => "ws",
            "https" => "wss",
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unsupported Send host scheme",
                )
                .into())
            }
        };
        ws_url
            .set_scheme(scheme)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid WebSocket URL"))?;
        let mut request = ws_url.as_str().into_client_request()?;
        request.headers_mut().insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static(WEBSOCKET_PROTOCOL),
        );

        // Proxy basic auth must also be present on the WebSocket upgrade request.
        if let Some((user, password)) = &self.config.basic_auth {
            let credential = format!("{}:{}", user, password.as_deref().unwrap_or(""));
            let value = format!("Basic {}", base64::encode(credential));
            request
                .headers_mut()
                .insert(header::AUTHORIZATION, HeaderValue::from_str(&value)?);
        }

        // Do not forward credentials across a redirect to another host.
        let (socket, response) = connect_with_config(request, None, 0)?;
        if response
            .headers()
            .get(header::SEC_WEBSOCKET_PROTOCOL)
            .and_then(|v| v.to_str().ok())
            != Some(WEBSOCKET_PROTOCOL)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Send WebSocket protocol was not accepted",
            )
            .into());
        }
        Ok(socket)
    }

    /// Configure the given reqwest client to match the configuration.
    fn configure(&self, mut client: ReqwestRequestBuilder) -> ReqwestRequestBuilder {
        // Configure basic HTTP authentication
        if let Some((user, password)) = &self.config.basic_auth {
            client = client.basic_auth(user, password.to_owned());
        }

        client
    }
}

/// Configurable properties for networking client used for ffsend actions.
#[derive(Clone, Debug, Builder)]
pub struct ClientConfig {
    /// Connection timeout for control requests.
    #[builder(default = "Some(Duration::from_secs(30))")]
    timeout: Option<Duration>,

    /// Connection timeout specific to file transfers.
    #[builder(default = "Some(Duration::from_secs(86400))")]
    transfer_timeout: Option<Duration>,

    /// Basic HTTP authentication credentials.
    ///
    /// Consists of a username, and an optional password.
    #[builder(default)]
    basic_auth: Option<(String, Option<String>)>,
    // TODO: proxy settings
}

impl ClientConfig {
    /// Construct a ffsend client based on this configuration.
    ///
    /// If this client is used for transfering files, `transfer` should be true.
    pub fn client(self, transfer: bool) -> Client {
        Client::new(self, transfer)
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: None,
            transfer_timeout: None,
            basic_auth: None,
        }
    }
}
