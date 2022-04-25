# Semio Client

A library providing Rust bindings to query a
[Semio Database](https://github.com/semio-ai/semio-db.git).

## Authentication

All operations rely on a [`Context`](src/context.rs),
which holds a [`reqwest::Client`](https://docs.rs/reqwest/latest/reqwest/struct.Client.html).
It must be configured with the right token in the HTTP header:

```rust
use reqwest::{
  header::{self, HeaderMap, HeaderValue},
  Client, Url,
};

let mut headers = HeaderMap::new();
headers.insert(
  header::AUTHORIZATION,
  HeaderValue::from_str(token.as_str())?,
);
let client = Client::builder().default_headers(headers).build().unwrap();
let context = Context::new(Url::parse("localhost:8080").unwrap(), client);
```

Credentials and tokens can be saved in a [`Config`](src/authentication.rs) object,
which can then be serialized into persistent storage.
**Pay attention to store them into a safe location.**

Tokens have a limited lifespan,
so it is recommended to refresh them at the beginning of a session by calling
[`access_token(config)`](src/authentication.rs).
It returns a suggested update of the config to apply, as a `ConfigMutation`.
For example:

```rust
use semio_client;

let mut config: Config; // load it prior to the following code.
let (token, config_mutation) =
  semio_client::authentication::access_token(&config)
    .await
    .expect("error while refreshing access token");
config = Some(config_mutation.next(config));
```

## Record and query types

The database provides records of type
[`User`](src/user.rs),
[`Organization`](src/organization.rs),
[`Folder`](src/folder.rs),
[`Enumeration`](src/enumeration.rs),
[`Structure`](src/structure.rs) and
[`Module`](src/module.rs).
Each type of record has a specific set of queries,
but some of them are common:
- `get_public`: gets the latest record in the database,
  omitting private information.
  The record may contain references to versions of its dependencies
  that must still be resolved.
- `get_private`: gets the latest record in the database,
  including private information.
  Requires the user to have certain permissions.
- `tagged`: gets a frozen record matching the given version tag.
  The record's references to its dependencies are already resolved,
  and should correspond to versions existing in the database.
- `tagged_req`: gets the latest frozen record matching
  the given version requirement tag.
- `tags`: lists the tags of a record.
  This can be used to resolve the version tag matching a version requirement.

## Selector

A [`Selector`](src/common.rs) is an identifier of a record,
either in a form of an [`UUID`](https://docs.rs/uuid/latest/uuid/index.html),
or of a dotted path, a `String` like `"my_user.my_folder.my_module"`
or `"my_organization.my_folder.MyStructureType"`.
