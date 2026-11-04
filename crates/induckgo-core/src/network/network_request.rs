//! One request a page is about to make, as the policies see it.

use url::Url;

use crate::network::resource_type::ResourceType;

#[derive(Debug, Clone)]
pub struct NetworkRequest {
    address: String,
    parsed: Option<Url>,
    first_party_host: Option<String>,
    resource_type: ResourceType,
    request_id: u64,
}

impl NetworkRequest {
    /// `first_party` is the address of the page that makes the request; `None` for address-bar navigations.
    /// An address that does not parse is kept as is and has no host.
    #[must_use]
    pub fn new(address: &str, first_party: Option<&str>, resource_type: ResourceType, request_id: u64) -> Self {
        let first_party_host = first_party
            .and_then(|page| Url::parse(page).ok())
            .and_then(|page| page.host_str().map(str::to_ascii_lowercase));
        Self {
            address: address.to_owned(),
            parsed: Url::parse(address).ok(),
            first_party_host,
            resource_type,
            request_id,
        }
    }

    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }

    #[must_use]
    pub fn host(&self) -> Option<&str> {
        self.parsed.as_ref().and_then(Url::host_str)
    }

    /// Host of the page making the request. `None` for navigations from the address bar and pop-ups.
    #[must_use]
    pub fn first_party_host(&self) -> Option<&str> {
        self.first_party_host.as_deref()
    }

    #[must_use]
    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    #[must_use]
    pub fn request_id(&self) -> u64 {
        self.request_id
    }

    /// `http`, `https`, `ws` and `wss` go over the network; `data:`, `blob:`, `file:` and the like do not.
    #[must_use]
    pub fn goes_over_the_network(&self) -> bool {
        self.parsed.as_ref().is_some_and(|url| matches!(url.scheme(), "http" | "https" | "ws" | "wss"))
    }

    #[must_use]
    pub fn is_plain_http(&self) -> bool {
        self.parsed.as_ref().is_some_and(|url| url.scheme() == "http")
    }

    /// True for the request that loads the page itself (typed address, clicked link), not its sub-resources.
    #[must_use]
    pub fn is_navigation(&self) -> bool {
        self.resource_type == ResourceType::MainFrame
    }
}
