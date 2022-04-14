use crate::{
  cli::{std_prompt, StdPromptOptions},
  context::Context,
  mutation::Mutation,
  user::{refresh_with_id, Logout, RefreshWithId},
  Token,
};
use chrono::Utc;
use clap::Parser;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
  pub url: Option<String>,
  pub access: Option<Token>,
  pub refresh: Option<Token>,
  pub user_id: Option<Uuid>,
}

impl Config {
  pub fn default() -> Self {
    Self {
      url: Some("localhost:8080".to_string()),
      access: None,
      refresh: None,
      user_id: None,
    }
  }
}

pub struct ConfigMutation {
  pub url: Mutation<String>,
  pub access: Mutation<Token>,
  pub refresh: Mutation<Token>,
  pub user_id: Mutation<Uuid>,
}

impl ConfigMutation {
  pub fn union(self, other: ConfigMutation) -> ConfigMutation {
    ConfigMutation {
      url: self.url.union(other.url),
      access: self.access.union(other.access),
      refresh: self.refresh.union(other.refresh),
      user_id: self.user_id.union(other.user_id),
    }
  }
  pub fn next(self, current: Config) -> Config {
    Config {
      url: self.url.next(current.url),
      access: self.access.next(current.access),
      refresh: self.refresh.next(current.refresh),
      user_id: self.user_id.next(current.user_id),
    }
  }
}

impl Default for ConfigMutation {
  fn default() -> Self {
    ConfigMutation {
      url: Mutation::None,
      access: Mutation::None,
      refresh: Mutation::None,
      user_id: Mutation::None,
    }
  }
}

/// Extract access token if available in a config, and refresh it if necessary.
/// Returns a config mutation that can be used to update the config.
pub async fn access_token(config: &Config) -> anyhow::Result<(Option<String>, ConfigMutation)> {
  let url = Url::parse(
    config
      .url
      .as_ref()
      .ok_or(anyhow::anyhow!("missing URL information in config"))?,
  )
  .map_err(|_| anyhow::anyhow!("Invalid registry URL"))?;

  if let Some(access) = &config.access {
    if access.expires_at > Utc::now() {
      Ok((Some(access.token.clone()), Default::default()))
    } else {
      if let Some(user_id) = &config.user_id {
        if let Some(refresh) = &config.refresh {
          let client = Client::builder().build()?;
          let context = Context::new(url, client);
          let res = refresh_with_id(
            &context,
            RefreshWithId {
              id: user_id.clone(),
              refresh_token: refresh.token.clone(),
            },
          )
          .await?;

          let mut mutation = ConfigMutation {
            ..Default::default()
          };

          let ret = res.access_token.token.clone();
          mutation.access = Mutation::Set(res.access_token);

          if let Some(refresh_token) = res.refresh_token {
            mutation.refresh = Mutation::Set(refresh_token);
          }

          mutation.user_id = Mutation::Set(res.id);

          Ok((Some(ret), mutation))
        } else {
          eprintln!("No refresh token found. Please run `semio user login`.");
          Ok((None, Default::default()))
        }
      } else {
        eprintln!("No user id found in configuration file");
        Ok((None, Default::default()))
      }
    }
  } else {
    Ok((None, Default::default()))
  }
}

#[derive(Debug, Parser)]
pub struct Login {
  #[clap(short, long, name = "user-name")]
  pub user_name: Option<String>,

  #[clap(short, long)]
  pub password: Option<String>,
}

/// Logins the user with the given credentials.
/// If a piece of login information is missing, requests it via the command line interface.
/// Returns a config mutation that can be used to update the config.
pub async fn login(context: &Context, login: Login) -> anyhow::Result<ConfigMutation> {
  let user_name = if let Some(user_name) = login.user_name {
    user_name
  } else {
    std_prompt(StdPromptOptions {
      echo: true,
      prompt: "User name".to_string(),
    })
    .await?
  };

  let password = if let Some(password) = login.password {
    password
  } else {
    std_prompt(StdPromptOptions {
      echo: false,
      prompt: "Password".to_string(),
    })
    .await?
  };

  let res = crate::user::login(
    context,
    crate::user::Login {
      user_name,
      password,
    },
  )
  .await?;

  let mut mutation = ConfigMutation {
    access: Mutation::Set(res.access_token),
    user_id: Mutation::Set(res.id),
    ..Default::default()
  };

  if let Some(refresh_token) = res.refresh_token {
    mutation.refresh = Mutation::Set(refresh_token);
  }

  Ok(mutation)
}

#[derive(Debug, Parser)]
pub struct Signup {
  #[clap(short, long, name = "user-name")]
  pub user_name: Option<String>,

  #[clap(short, long)]
  pub first_name: Option<String>,

  #[clap(short, long)]
  pub last_name: Option<String>,

  #[clap(short, long)]
  pub email: Option<String>,

  #[clap(short, long)]
  pub password: Option<String>,
}

/// Signs up the user with the given credentials.
/// If a piece of login information is missing, requests it via the command line interface.
/// Returns a config mutation that can be used to update the config.
pub async fn signup(context: &Context, signup: Signup) -> anyhow::Result<ConfigMutation> {
  let user_name = if let Some(user_name) = signup.user_name {
    user_name
  } else {
    std_prompt(StdPromptOptions {
      echo: true,
      prompt: "User name".to_string(),
    })
    .await?
  };

  let first_name = if let Some(first_name) = signup.first_name {
    first_name
  } else {
    std_prompt(StdPromptOptions {
      echo: true,
      prompt: "First name".to_string(),
    })
    .await?
  };

  let last_name = if let Some(last_name) = signup.last_name {
    last_name
  } else {
    std_prompt(StdPromptOptions {
      echo: true,
      prompt: "Last name".to_string(),
    })
    .await?
  };

  let email = if let Some(email) = signup.email {
    email
  } else {
    std_prompt(StdPromptOptions {
      echo: true,
      prompt: "Email".to_string(),
    })
    .await?
  };

  let password = if let Some(password) = signup.password {
    password
  } else {
    std_prompt(StdPromptOptions {
      echo: false,
      prompt: "Password".to_string(),
    })
    .await?
  };

  let res = crate::user::signup(
    context,
    crate::user::Signup {
      user_name,
      first_name,
      last_name,
      email,
      password,
    },
  )
  .await?;

  let mut mutation = ConfigMutation {
    access: Mutation::Set(res.access_token),
    user_id: Mutation::Set(res.id),
    ..Default::default()
  };

  if let Some(refresh_token) = res.refresh_token {
    mutation.refresh = Mutation::Set(refresh_token);
  }

  Ok(mutation)
}

pub async fn logout(_: &Context, _: Logout) -> anyhow::Result<ConfigMutation> {
  Ok(ConfigMutation {
    access: Mutation::Unset,
    refresh: Mutation::Unset,
    user_id: Mutation::Unset,
    ..Default::default()
  })
}
