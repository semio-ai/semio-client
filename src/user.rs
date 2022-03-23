use chrono::DateTime;
use graphql_client::GraphQLQuery;
use clap::Parser;
use semio_record::{record::RecordDefn, user::v0::User};
use uuid::Uuid;

use crate::{context::Context, Token, common::{check_errors, Selector, FromResponse}};

macro_rules! impl_public_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <User as RecordDefn>::Public {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          user_name: value.user_name,
        })
      }
    }
  }
}

macro_rules! impl_private_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <User as RecordDefn>::Private {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          user_name: value.user_name,
          email: value.email,
          first_name: value.first_name,
          last_name: value.last_name,
          email_verified: value.email_verified,
        })
      }
    }
  }
}

impl_public_from_response!(get_public_query::GetPublicQueryUserGetPublic);
impl_private_from_response!(get_private_query::GetPrivateQueryUserGetPrivate);
impl_public_from_response!(get_version_public_query::GetVersionPublicQueryUserGetVersionPublic);
impl_private_from_response!(get_version_private_query::GetVersionPrivateQueryUserGetVersionPrivate);
impl_private_from_response!(login_query::LoginQueryUserLoginUser);
impl_private_from_response!(signup_query::SignupQueryUserSignupUser);
impl_private_from_response!(refresh_with_id_query::RefreshWithIdQueryUserRefreshWithIdUser);

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/user/login.graphql",
  response_derives = "Debug"
)]
pub struct LoginQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/user/login_with_id.graphql",
  response_derives = "Debug"
)]
pub struct LoginWithIdQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/user/signup.graphql",
  response_derives = "Debug"
)]
pub struct SignupQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/user/refresh_with_id.graphql",
  response_derives = "Debug"
)]
pub struct RefreshWithIdQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/user/set_user_name.graphql",
  response_derives = "Debug"
)]
pub struct SetUserNameQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/user/set_first_name.graphql",
  response_derives = "Debug"
)]
pub struct SetFirstNameQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/user/get_public.graphql",
  response_derives = "Debug,Serialize"
)]
pub struct GetPublicQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/user/get_private.graphql",
  response_derives = "Debug,Serialize"
)]
pub struct GetPrivateQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/user/get_version_public.graphql",
  response_derives = "Debug,Serialize"
)]
pub struct GetVersionPublicQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/user/get_version_private.graphql",
  response_derives = "Debug,Serialize"
)]
pub struct GetVersionPrivateQuery;

#[derive(Debug, Parser)]
pub struct Login {
  #[clap(short, long, name = "user-name")]
  pub user_name: String,

  #[clap(short, long)]
  pub password: String,
}


#[derive(Debug, Parser)]
pub struct Signup {
  #[clap(short, long, name = "user-name")]
  pub user_name: String,

  #[clap(short, long)]
  pub first_name: String,

  #[clap(short, long)]
  pub last_name: String,

  #[clap(short, long)]
  pub email: String,

  #[clap(short, long)]
  pub password: String,
}

#[derive(Debug, Parser)]
pub struct Logout {}

#[derive(Debug, Parser)]
pub struct SetUserName {
  pub user_name: String,
}

impl From<SetUserName> for set_user_name_query::Variables {
  fn from(value: SetUserName) -> Self {
    Self {
      user_name: value.user_name,
    }
  }
}

#[derive(Debug, Parser)]
pub struct SetFirstName {
  pub first_name: String,
}

impl From<SetFirstName> for set_first_name_query::Variables {
  fn from(value: SetFirstName) -> Self {
    Self {
      first_name: value.first_name,
    }
  }
}

#[derive(Debug, Parser)]
pub struct GetPublic {
  pub selector: Selector,
}

#[derive(Debug, Parser)]
pub struct GetPrivate {
}

#[derive(Debug, Parser)]
pub struct GetVersionPublic {
  pub selector: Selector,
  pub version: i64,
}

#[derive(Debug, Parser)]
pub struct GetVersionPrivate {
  pub version: i64,
}

impl From<GetVersionPrivate> for get_version_private_query::Variables {
  fn from(value: GetVersionPrivate) -> Self {
    Self {
      version: value.version
    }
  }
}

pub struct LoginResult {
  pub access_token: Token,
  pub refresh_token: Option<Token>,
  pub id: Uuid,
  pub user: <User as RecordDefn>::Private,
}

pub async fn login(context: &Context, login: Login) -> anyhow::Result<LoginResult> {
  let response = context.request::<LoginQuery>(login_query::Variables {
    user_name: login.user_name,
    password: login.password,
  }).await?;

  check_errors(&response.errors)?;

  let data = response.data.unwrap().user.login;

  Ok(LoginResult {
    access_token: Token {
      token: data.access.token,
      expires_at: DateTime::parse_from_rfc3339(&data.access.expires_at)?.into(),
    },
    refresh_token: data.refresh.and_then(|refresh| Some(Token {
      token: refresh.token,
      expires_at: DateTime::parse_from_rfc3339(&refresh.expires_at).ok()?.into(),
    })),
    id: data.id,
    user: <User as RecordDefn>::Private::from_response(data.user)?,
  })
}

pub struct SignupResult {
  pub access_token: Token,
  pub refresh_token: Option<Token>,
  pub id: Uuid,
  pub user: <User as RecordDefn>::Private,
}

pub async fn signup(context: &Context, signup: Signup) -> anyhow::Result<SignupResult> {
  let response = context.request::<SignupQuery>(signup_query::Variables {
    user_name: signup.user_name,
    first_name: signup.first_name,
    last_name: signup.last_name,
    email: signup.email,
    password: signup.password,
  }).await?;

  check_errors(&response.errors)?;

  let data = response.data.unwrap().user.signup;

  Ok(SignupResult {
    access_token: Token {
      token: data.access.token,
      expires_at: DateTime::parse_from_rfc3339(&data.access.expires_at)?.into(),
    },
    refresh_token: data.refresh.and_then(|refresh| Some(Token {
      token: refresh.token,
      expires_at: DateTime::parse_from_rfc3339(&refresh.expires_at).ok()?.into(),
    })),
    id: data.id,
    user: <User as RecordDefn>::Private::from_response(data.user)?,
  })
}

pub async fn set_user_name(context: &Context, set_user_name: SetUserName) -> anyhow::Result<i64> {
  let response = context.request::<SetUserNameQuery>(set_user_name.into()).await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().user.set_user_name)
}

pub async fn set_first_name(context: &Context, set_first_name: SetFirstName) -> anyhow::Result<i64> {
  let response = context.request::<SetFirstNameQuery>(
    set_first_name_query::Variables {
      first_name: set_first_name.first_name.clone(),
    },
  ).await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().user.set_first_name)
}

pub async fn get_public(context: &Context, get_public: GetPublic) -> anyhow::Result<<User as RecordDefn>::Public> {
  let id = get_public.selector.resolve(context).await?;
  let response = context.request::<GetPublicQuery>(get_public_query::Variables {
    id
  }).await?;

  check_errors(&response.errors)?;

  <User as RecordDefn>::Public::from_response(response.data.unwrap().user.get_public)
}

pub async fn get_private(context: &Context, _: GetPrivate) -> anyhow::Result<<User as RecordDefn>::Private> {
  let response = context.request::<GetPrivateQuery>(get_private_query::Variables {}).await?;

  check_errors(&response.errors)?;

  <User as RecordDefn>::Private::from_response(response.data.unwrap().user.get_private)
}

pub async fn get_version_public(context: &Context, get_version_public: GetVersionPublic) -> anyhow::Result<<User as RecordDefn>::Public> {
  let id = get_version_public.selector.resolve(context).await?;

  let response = context.request::<GetVersionPublicQuery>(get_version_public_query::Variables {
    id,
    version: get_version_public.version,
  }).await?;

  check_errors(&response.errors)?;

  <User as RecordDefn>::Public::from_response(response.data.unwrap().user.get_version_public)
}

pub async fn get_version_private(context: &Context, get_version_private: GetVersionPrivate) -> anyhow::Result<<User as RecordDefn>::Private> {
  let response = context.request::<GetVersionPrivateQuery>(get_version_private.into()).await?;

  check_errors(&response.errors)?;

  <User as RecordDefn>::Private::from_response(response.data.unwrap().user.get_version_private)
}

#[derive(Debug, Parser)]
pub struct RefreshWithId {
  pub id: Uuid,
  pub refresh_token: String,
}

pub struct RefreshWithIdResult {
  pub access_token: Token,
  pub refresh_token: Option<Token>,
  pub id: Uuid,
}

pub async fn refresh_with_id(context: &Context, data: RefreshWithId) -> anyhow::Result<LoginResult> {
  let response = context.request::<RefreshWithIdQuery>(refresh_with_id_query::Variables {
    id: data.id,
    refresh_token: data.refresh_token,
  }).await?;

  check_errors(&response.errors)?;

  let data = response.data.unwrap().user.refresh_with_id;

  Ok(LoginResult {
    access_token: Token {
      token: data.access.token,
      expires_at: DateTime::parse_from_rfc3339(&data.access.expires_at)?.into(),
    },
    refresh_token: data.refresh.and_then(|refresh| Some(Token {
      token: refresh.token,
      expires_at: DateTime::parse_from_rfc3339(&refresh.expires_at).ok()?.into(),
    })),
    id: data.id,
    user: <User as RecordDefn>::Private::from_response(data.user)?,
  })
}