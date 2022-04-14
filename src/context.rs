use graphql_client::{GraphQLQuery, Response};
use reqwest::{Client, Url};

/// An HTTP context to query the Semio API.
pub struct Context {
  /// The part of the URL with the host, port, and the base path (typically `/`) of the HTTP endpoint.
  pub url: String,

  /// An HTTP client. Consider setting the `AUTHORIZATION` header with the access token for authentication.
  pub client: Client,
}

impl Context {
  pub fn new(url: Url, client: Client) -> Self {
    Self {
      url: format!(
        "{}:{}{}",
        url.host_str().unwrap(),
        url.port().unwrap(),
        url.path()
      ),
      client,
    }
  }
}

impl Context {
  pub async fn request<Q: GraphQLQuery>(
    &self,
    variables: Q::Variables,
  ) -> Result<Response<Q::ResponseData>, reqwest::Error> {
    let body = Q::build_query(variables);
    let reqwest_response = self
      .client
      .post(format!("http://{}graphql", self.url))
      .json(&body)
      .send()
      .await?;
    Ok(reqwest_response.json().await?)
  }
}
