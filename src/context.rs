use graphql_client::{GraphQLQuery, Response};
use reqwest::{Client, Url};

pub struct Context {
  pub url: String,
  pub client: Client,
}

impl Context {
  pub fn new(url: Url, client: Client) -> Self {
    Self {
      url: format!(
        "{}:{}/{}",
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
      .post(format!("http://{}/graphql", self.url))
      .json(&body)
      .send()
      .await?;
    Ok(reqwest_response.json().await?)
  }
}
