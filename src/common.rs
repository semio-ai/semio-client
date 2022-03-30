use std::{
  fmt::{Display, Formatter},
  str::FromStr,
};

use clap::Parser;
use graphql_client::GraphQLQuery;
use semio_record::{
  record::{UnfrozenReference, Version, VersionReq},
  ty::{Primitive, PrimitiveKind, UnfrozenArray, UnfrozenScalar, UnfrozenTy},
};
use std::fmt::Write;
use uuid::Uuid;

use crate::context::Context;
#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/common/lookup.graphql",
  response_derives = "Debug"
)]
pub struct LookupQuery;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Selector {
  Id(Uuid),
  Path(String),
}

#[derive(Debug)]
pub struct SelectorVersionReq {
  pub selector: Selector,
  pub version_req: VersionReq,
}

#[derive(Debug)]
pub enum UnfrozenSelectorTy {
  Primitive(Primitive),
  Scalar(SelectorVersionReq),
  Array(SelectorVersionReq),
}

impl From<PrimitiveKind> for UnfrozenSelectorTy {
  fn from(kind: PrimitiveKind) -> Self {
    Self::Primitive(kind.into())
  }
}

pub trait FromResponse<T>: Sized {
  fn from_response(res: T) -> anyhow::Result<Self>;
}

impl FromStr for UnfrozenSelectorTy {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let array = s.ends_with("[]");
    let s = if array { &s[..s.len() - 2] } else { s };
    let mut iter = s.split('@');
    let s = iter.next().ok_or("Expected type".to_string())?;

    match s {
      "unit" => Ok(PrimitiveKind::Unit.into()),
      "bool" => Ok(
        if !array {
          PrimitiveKind::Boolean
        } else {
          PrimitiveKind::ArrayBoolean
        }
        .into(),
      ),
      "u8" => Ok(
        if !array {
          PrimitiveKind::U8
        } else {
          PrimitiveKind::ArrayU8
        }
        .into(),
      ),
      "u16" => Ok(
        if !array {
          PrimitiveKind::U16
        } else {
          PrimitiveKind::ArrayU16
        }
        .into(),
      ),
      "u32" => Ok(
        if !array {
          PrimitiveKind::U32
        } else {
          PrimitiveKind::ArrayU32
        }
        .into(),
      ),
      "u64" => Ok(
        if !array {
          PrimitiveKind::U64
        } else {
          PrimitiveKind::ArrayU64
        }
        .into(),
      ),
      "i8" => Ok(
        if !array {
          PrimitiveKind::I8
        } else {
          PrimitiveKind::ArrayI8
        }
        .into(),
      ),
      "i16" => Ok(
        if !array {
          PrimitiveKind::I16
        } else {
          PrimitiveKind::ArrayI16
        }
        .into(),
      ),
      "i32" => Ok(
        if !array {
          PrimitiveKind::I32
        } else {
          PrimitiveKind::ArrayI32
        }
        .into(),
      ),
      "i64" => Ok(
        if !array {
          PrimitiveKind::I64
        } else {
          PrimitiveKind::ArrayI64
        }
        .into(),
      ),
      "f32" => Ok(
        if !array {
          PrimitiveKind::F32
        } else {
          PrimitiveKind::ArrayF32
        }
        .into(),
      ),
      "f64" => Ok(
        if !array {
          PrimitiveKind::F64
        } else {
          PrimitiveKind::ArrayF64
        }
        .into(),
      ),
      "str" => Ok(
        if !array {
          PrimitiveKind::String
        } else {
          PrimitiveKind::ArrayString
        }
        .into(),
      ),
      _ => {
        let selector = Selector::from_str(s)?;
        let version_req = VersionReq(
          iter
            .next()
            .map(|v| semver::VersionReq::parse(v).ok())
            .flatten(),
        );
        let reference = SelectorVersionReq {
          selector,
          version_req,
        };
        Ok(if array {
          Self::Array(reference)
        } else {
          Self::Scalar(reference)
        })
      }
    }
  }
}

impl FromStr for Selector {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if let Ok(id) = Uuid::parse_str(s) {
      Ok(Self::Id(id))
    } else {
      Ok(Self::Path(s.to_string()))
    }
  }
}

impl Display for Selector {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Id(id) => write!(f, "{}", id),
      Self::Path(path) => write!(f, "{}", path),
    }
  }
}

#[derive(Debug, Parser)]
pub struct Create {
  pub parent: Selector,
  pub name: String,
}

#[derive(Debug, Parser)]
pub struct SetName {
  pub id: Selector,
  pub name: String,
}

#[derive(Debug, Parser)]
pub struct SetParent {
  pub id: Selector,
  pub parent_id: Selector,
}

#[derive(Debug, Parser)]
pub struct AddPermissions {
  pub id: Selector,
  pub agent_id: Selector,
  pub with_permissions: String,
}

#[derive(Debug, Parser)]
pub struct SetPermissions {
  pub id: Selector,
  pub agent_id: Selector,
  pub with_permissions: String,
}

#[derive(Debug, Parser)]
pub struct RemovePermissions {
  pub id: Selector,
  pub agent_id: Selector,
}

#[derive(Debug, Parser)]
pub struct SetDefaultPermissions {
  pub id: Selector,
  pub with_permissions: String,
}

#[derive(Debug, Parser)]
pub struct GetPublic {
  pub id: Selector,
}

#[derive(Debug, Parser)]
pub struct GetPrivate {
  pub id: Selector,
}

#[derive(Debug, Parser)]
pub struct GetVersionPublic {
  pub id: Selector,
  pub version: i64,
}

#[derive(Debug, Parser)]
pub struct GetVersionPrivate {
  pub id: Selector,
  pub version: i64,
}

#[derive(Debug, Parser)]
pub struct Tag {
  pub selector: Selector,
  pub version: String,
}

#[derive(Debug, Parser)]
pub struct Tagged {
  pub selector: Selector,
  pub version: String,
}

#[derive(Debug, Parser)]
pub struct TaggedReq {
  pub selector: Selector,
  pub version_req: String,
}

#[derive(Debug, Parser)]
pub struct GetFrozen {
  pub selector: Selector,
}

#[derive(Debug, Parser)]
pub struct GetVersionFrozen {
  pub selector: Selector,
  pub version: i64,
}

impl Selector {
  pub async fn resolve<'a>(&self, context: &Context) -> anyhow::Result<Uuid> {
    match self {
      Self::Id(id) => Ok(id.clone()),
      Self::Path(path) => {
        let response = context
          .request::<LookupQuery>(lookup_query::Variables { path: path.clone() })
          .await?;

        if let Some(errors) = response.errors {
          for error in errors {
            eprintln!("{}", error);
          }

          return Err(anyhow::anyhow!("lookup failed"));
        }

        Ok(response.data.unwrap().lookup)
      }
    }
  }
}

impl UnfrozenSelectorTy {
  pub async fn resolve<'a>(&self, context: &Context) -> anyhow::Result<UnfrozenTy> {
    match self {
      Self::Primitive(primitive) => Ok(primitive.kind.clone().into()),
      Self::Scalar(reference) => {
        let id = reference.selector.resolve(context).await?;
        Ok(UnfrozenTy::UnfrozenScalar(UnfrozenScalar {
          reference: UnfrozenReference {
            id,
            version_req: reference.version_req.clone(),
          },
        }))
      }
      Self::Array(reference) => {
        let id = reference.selector.resolve(context).await?;
        Ok(UnfrozenTy::UnfrozenArray(UnfrozenArray {
          reference: UnfrozenReference {
            id,
            version_req: reference.version_req.clone(),
          },
        }))
      }
    }
  }
}

pub fn check_errors(errors: &Option<Vec<graphql_client::Error>>) -> anyhow::Result<()> {
  if let Some(errors) = errors {
    let mut message = String::new();
    for error in errors {
      writeln!(message, "{}", error)?;
    }

    return Err(anyhow::anyhow!(message));
  }

  Ok(())
}

macro_rules! impl_get_public {
  ($res: ty, $id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug,Serialize,Deserialize"
                        )]
    pub struct GetPublicQuery;

    pub async fn get_public<'a>(
      context: &crate::context::Context,
      data: crate::common::GetPublic,
    ) -> anyhow::Result<$res> {
      let id = data.id.resolve(context).await?;
      let response = context
        .request::<GetPublicQuery>(get_public_query::Variables { id })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(<$res>::from_response(
        response.data.unwrap().$id.get_public,
      )?)
    }
  };
}

pub(crate) use impl_get_public;

macro_rules! impl_get_private {
  ($res: ty, $id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug,Serialize,Deserialize"
                        )]
    pub struct GetPrivateQuery;

    pub async fn get_private(
      context: &crate::context::Context,
      data: crate::common::GetPrivate,
    ) -> anyhow::Result<$res> {
      let id = data.id.resolve(context).await?;
      let response = context
        .request::<GetPrivateQuery>(get_private_query::Variables { id })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(<$res>::from_response(
        response.data.unwrap().$id.get_private,
      )?)
    }
  };
}

pub(crate) use impl_get_private;

macro_rules! impl_get_version_public {
  ($res: ty, $id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug,Serialize,Deserialize"
                        )]
    pub struct GetVersionPublicQuery;

    pub async fn get_version_public<'a>(
      context: &crate::context::Context,
      data: crate::common::GetVersionPublic,
    ) -> anyhow::Result<$res> {
      let id = data.id.resolve(context).await?;
      let response = context
        .request::<GetVersionPublicQuery>(get_version_public_query::Variables {
          id,
          version: data.version,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(<$res>::from_response(
        response.data.unwrap().$id.get_version_public,
      )?)
    }
  };
}

pub(crate) use impl_get_version_public;

macro_rules! impl_get_version_private {
  ($res: ty, $id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug,Serialize,Deserialize"
                        )]
    pub struct GetVersionPrivateQuery;

    pub async fn get_version_private<'a>(
      context: &crate::context::Context,
      data: crate::common::GetVersionPrivate,
    ) -> anyhow::Result<$res> {
      let id = data.id.resolve(context).await?;
      let response = context
        .request::<GetVersionPrivateQuery>(get_version_private_query::Variables {
          id,
          version: data.version,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(<$res>::from_response(
        response.data.unwrap().$id.get_version_private,
      )?)
    }
  };
}

pub(crate) use impl_get_version_private;

macro_rules! impl_set_name {
  ($id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug"
                        )]
    pub struct SetNameQuery;

    pub async fn set_name<'a>(
      context: &crate::context::Context,
      data: crate::common::SetName,
    ) -> anyhow::Result<i64> {
      let id = data.id.resolve(context).await?;
      let response = context
        .request::<SetNameQuery>(set_name_query::Variables {
          id,
          name: data.name,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(response.data.unwrap().$id.set_name)
    }
  };
}

pub(crate) use impl_set_name;

macro_rules! impl_set_parent {
  ($id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug"
                        )]
    pub struct SetParentQuery;

    pub async fn set_parent<'a>(
      context: &crate::context::Context,
      data: crate::common::SetParent,
    ) -> anyhow::Result<i64> {
      let id = data.id.resolve(context).await?;
      let parent = data.parent_id.resolve(context).await?;

      let response = context
        .request::<SetParentQuery>(set_parent_query::Variables { id, parent })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(response.data.unwrap().$id.set_parent)
    }
  };
}

pub(crate) use impl_set_parent;

macro_rules! impl_add_permissions {
  ($id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug"
                        )]
    pub struct AddPermissionsQuery;

    pub async fn add_permissions<'a>(
      context: &crate::context::Context,
      data: crate::common::AddPermissions,
    ) -> anyhow::Result<i64> {
      let id = data.id.resolve(context).await?;
      let agent = data.agent_id.resolve(context).await?;

      let response = context
        .request::<AddPermissionsQuery>(add_permissions_query::Variables {
          id,
          agent_id: agent,
          with_permissions: data.with_permissions,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(response.data.unwrap().$id.add_permissions)
    }
  };
}

pub(crate) use impl_add_permissions;

macro_rules! impl_set_permissions {
  ($id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug"
                        )]
    pub struct SetPermissionsQuery;

    pub async fn set_permissions<'a>(
      context: &crate::context::Context,
      data: crate::common::SetPermissions,
    ) -> anyhow::Result<i64> {
      let id = data.id.resolve(context).await?;
      let agent = data.agent_id.resolve(context).await?;

      let response = context
        .request::<SetPermissionsQuery>(set_permissions_query::Variables {
          id,
          agent_id: agent,
          with_permissions: data.with_permissions,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(response.data.unwrap().$id.set_permissions)
    }
  };
}

pub(crate) use impl_set_permissions;

macro_rules! impl_remove_permissions {
  ($id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug"
                        )]
    pub struct RemovePermissionsQuery;

    pub async fn remove_permissions<'a>(
      context: &crate::context::Context,
      data: crate::common::RemovePermissions,
    ) -> anyhow::Result<i64> {
      let id = data.id.resolve(context).await?;
      let agent = data.agent_id.resolve(context).await?;

      let response = context
        .request::<RemovePermissionsQuery>(remove_permissions_query::Variables {
          id,
          agent_id: agent,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(response.data.unwrap().$id.remove_permissions)
    }
  };
}

pub(crate) use impl_remove_permissions;

macro_rules! impl_set_default_permissions {
  ($id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug"
                        )]
    pub struct SetDefaultPermissionsQuery;

    pub async fn set_default_permissions<'a>(
      context: &crate::context::Context,
      data: crate::common::SetDefaultPermissions,
    ) -> anyhow::Result<i64> {
      let id = data.id.resolve(context).await?;

      let response = context
        .request::<SetDefaultPermissionsQuery>(set_default_permissions_query::Variables {
          id,
          with_permissions: data.with_permissions,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(response.data.unwrap().$id.set_default_permissions)
    }
  };
}

pub(crate) use impl_set_default_permissions;

macro_rules! impl_create {
  ($id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug"
                        )]
    pub struct CreateQuery;

    pub async fn create(
      context: &crate::context::Context,
      data: crate::common::Create,
    ) -> anyhow::Result<uuid::Uuid> {
      let parent = data.parent.resolve(context).await?;
      let response = context
        .request::<CreateQuery>(create_query::Variables {
          parent,
          name: data.name,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(response.data.unwrap().$id.create)
    }
  };
}

pub(crate) use impl_create;

macro_rules! impl_tag {
  ($path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug"
                        )]
    pub struct TagQuery;

    pub async fn tag<'a>(
      context: &crate::context::Context,
      data: crate::common::Tag,
    ) -> anyhow::Result<()> {
      let id = data.selector.resolve(context).await?;
      let response = context
        .request::<TagQuery>(tag_query::Variables {
          id,
          version: data.version,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      Ok(())
    }
  };
}

pub(crate) use impl_tag;

macro_rules! impl_tagged {
  ($res: ty, $id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug,Serialize"
                        )]
    pub struct TaggedQuery;

    pub async fn tagged<'a>(
      context: &crate::context::Context,
      data: crate::common::Tagged,
    ) -> anyhow::Result<$res> {
      let id = data.selector.resolve(context).await?;
      let response = context
        .request::<TaggedQuery>(tagged_query::Variables {
          id,
          version: data.version,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      <$res>::from_response(response.data.unwrap().$id.tagged)
    }
  };
}

pub(crate) use impl_tagged;

macro_rules! impl_tagged_req {
  ($res: ty, $id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug,Serialize"
                        )]
    pub struct TaggedReqQuery;

    pub async fn tagged_req<'a>(
      context: &crate::context::Context,
      data: crate::common::TaggedReq,
    ) -> anyhow::Result<$res> {
      let id = data.selector.resolve(context).await?;
      let response = context
        .request::<TaggedReqQuery>(tagged_req_query::Variables {
          id,
          version_req: data.version_req,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      <$res>::from_response(response.data.unwrap().$id.tagged_req)
    }
  };
}

pub(crate) use impl_tagged_req;

macro_rules! impl_get_frozen {
  ($res: ty, $id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug,Serialize"
                        )]
    pub struct GetFrozenQuery;

    pub async fn get_frozen<'a>(
      context: &crate::context::Context,
      data: crate::common::GetFrozen,
    ) -> anyhow::Result<$res> {
      let id = data.selector.resolve(context).await?;
      let response = context
        .request::<GetFrozenQuery>(get_frozen_query::Variables {
          id,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      <$res>::from_response(response.data.unwrap().$id.get_frozen)
    }
  };
}

pub(crate) use impl_get_frozen;

macro_rules! impl_get_version_frozen {
  ($res: ty, $id: tt, $path: literal) => {
    #[derive(graphql_client::GraphQLQuery)]
    #[graphql(
                          schema_path = "src/schema.graphql.json",
                          query_path = $path,
                          response_derives = "Debug,Serialize"
                        )]
    pub struct GetVersionFrozenQuery;

    pub async fn get_version_frozen<'a>(
      context: &crate::context::Context,
      data: crate::common::GetVersionFrozen,
    ) -> anyhow::Result<$res> {
      let id = data.selector.resolve(context).await?;
      let response = context
        .request::<GetVersionFrozenQuery>(get_version_frozen_query::Variables {
          id,
          version: data.version,
        })
        .await?;

      crate::common::check_errors(&response.errors)?;

      <$res>::from_response(response.data.unwrap().$id.get_version_frozen)
    }
  };
}

pub(crate) use impl_get_version_frozen;


macro_rules! impl_permission_level_from_response {
  ($query: path) => {
    impl crate::common::FromResponse<$query> for semio_record::acl::PermissionLevel {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        match value {
          <$query>::NONE => Ok(semio_record::acl::PermissionLevel::None),
          <$query>::PUBLIC => Ok(semio_record::acl::PermissionLevel::Public),
          <$query>::PRIVATE => Ok(semio_record::acl::PermissionLevel::Private),
          _ => Err(anyhow::anyhow!("unexpected permission level {:?}", value)),
        }
      }
    }
  };
}

pub(crate) use impl_permission_level_from_response;

macro_rules! impl_primitive_kind_from_response {
  ($query: path) => {
    impl crate::common::FromResponse<$query> for semio_record::ty::PrimitiveKind {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        match value {
          <$query>::ARRAY_BOOLEAN => Ok(semio_record::ty::PrimitiveKind::ArrayBoolean),
          <$query>::ARRAY_F32 => Ok(semio_record::ty::PrimitiveKind::ArrayF32),
          <$query>::ARRAY_F64 => Ok(semio_record::ty::PrimitiveKind::ArrayF64),
          <$query>::ARRAY_U8 => Ok(semio_record::ty::PrimitiveKind::ArrayU8),
          <$query>::ARRAY_U16 => Ok(semio_record::ty::PrimitiveKind::ArrayU16),
          <$query>::ARRAY_U32 => Ok(semio_record::ty::PrimitiveKind::ArrayU32),
          <$query>::ARRAY_U64 => Ok(semio_record::ty::PrimitiveKind::ArrayU64),
          <$query>::ARRAY_I8 => Ok(semio_record::ty::PrimitiveKind::ArrayI8),
          <$query>::ARRAY_I16 => Ok(semio_record::ty::PrimitiveKind::ArrayI16),
          <$query>::ARRAY_I32 => Ok(semio_record::ty::PrimitiveKind::ArrayI32),
          <$query>::ARRAY_I64 => Ok(semio_record::ty::PrimitiveKind::ArrayI64),
          <$query>::ARRAY_STRING => Ok(semio_record::ty::PrimitiveKind::ArrayString),
          <$query>::BOOLEAN => Ok(semio_record::ty::PrimitiveKind::Boolean),
          <$query>::F32 => Ok(semio_record::ty::PrimitiveKind::F32),
          <$query>::F64 => Ok(semio_record::ty::PrimitiveKind::F64),
          <$query>::I8 => Ok(semio_record::ty::PrimitiveKind::I8),
          <$query>::I16 => Ok(semio_record::ty::PrimitiveKind::I16),
          <$query>::I32 => Ok(semio_record::ty::PrimitiveKind::I32),
          <$query>::I64 => Ok(semio_record::ty::PrimitiveKind::I64),
          <$query>::STRING => Ok(semio_record::ty::PrimitiveKind::String),
          <$query>::U8 => Ok(semio_record::ty::PrimitiveKind::U8),
          <$query>::U16 => Ok(semio_record::ty::PrimitiveKind::U16),
          <$query>::U32 => Ok(semio_record::ty::PrimitiveKind::U32),
          <$query>::U64 => Ok(semio_record::ty::PrimitiveKind::U64),
          <$query>::UNIT => Ok(semio_record::ty::PrimitiveKind::Unit),
          _ => Err(anyhow::anyhow!("unexpected primitive kind {:?}", value)),
        }
      }
    }
  };
}

pub(crate) use impl_primitive_kind_from_response;

macro_rules! impl_primitive_from_response {
  ($query: path) => {
    impl FromResponse<$query> for semio_record::ty::Primitive {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          kind: semio_record::ty::PrimitiveKind::from_response(value.kind)?,
        })
      }
    }
  };
}

pub(crate) use impl_primitive_from_response;

macro_rules! impl_unfrozen_ty_from_response {
  ($query: path) => {
    impl FromResponse<$query> for semio_record::ty::UnfrozenTy {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        match value {
          <$query>::Primitive(value) => Ok(Self::Primitive(
            semio_record::ty::Primitive::from_response(value)?,
          )),
          <$query>::UnfrozenScalar(value) => {
            Ok(Self::UnfrozenScalar(semio_record::ty::UnfrozenScalar {
              reference: semio_record::record::UnfrozenReference {
                id: value.reference.id,
                version_req: value.reference.version_req,
              },
            }))
          }
          <$query>::UnfrozenArray(value) => {
            Ok(Self::UnfrozenArray(semio_record::ty::UnfrozenArray {
              reference: semio_record::record::UnfrozenReference {
                id: value.reference.id,
                version_req: value.reference.version_req,
              },
            }))
          }
        }
      }
    }
  };
}

pub(crate) use impl_unfrozen_ty_from_response;

macro_rules! impl_frozen_ty_from_response {
  ($query: path) => {
    impl FromResponse<$query> for semio_record::ty::FrozenTy {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        match value {
          <$query>::Primitive(value) => Ok(Self::Primitive(
            semio_record::ty::Primitive::from_response(value)?,
          )),
          <$query>::FrozenScalar(value) => Ok(Self::FrozenScalar(semio_record::ty::FrozenScalar {
            reference: semio_record::record::FrozenReference {
              id: value.reference.id,
              version: value.reference.version,
            },
          })),
          <$query>::FrozenArray(value) => Ok(Self::FrozenArray(semio_record::ty::FrozenArray {
            reference: semio_record::record::FrozenReference {
              id: value.reference.id,
              version: value.reference.version,
            },
          })),
        }
      }
    }
  };
}

pub(crate) use impl_frozen_ty_from_response;

macro_rules! impl_with_permissions_from_response {
  ($query: path) => {
    impl FromResponse<$query> for semio_record::acl::WithPermissions {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        match value {
          <$query>::Inherit(inherit) => Ok(Self::Inherit(semio_record::acl::Inherit {
            from: inherit.from,
          })),
          <$query>::Permissions(custom) => Ok(Self::Custom(semio_record::acl::Permissions {
            read: semio_record::acl::PermissionLevel::from_response(custom.read)?,
            write: semio_record::acl::PermissionLevel::from_response(custom.write)?,
          })),
          _ => Ok(Self::None(semio_record::acl::None { _dummy: 0 })),
        }
      }
    }
  };
}

pub(crate) use impl_with_permissions_from_response;

macro_rules! impl_acl_from_response {
  ($query: path) => {
    impl FromResponse<$query> for semio_record::acl::Acl {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut permissions = std::collections::HashMap::new();
        for id_permission in value.permissions {
          permissions.insert(
            id_permission.id,
            semio_record::acl::WithPermissions::from_response(id_permission.with_permissions)?,
          );
        }
        Ok(Self {
          permissions,
          default: semio_record::acl::WithPermissions::from_response(value.default)?,
        })
      }
    }
  };
}

pub(crate) use impl_acl_from_response;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/common/tags.graphql",
  response_derives = "Debug"
)]
pub struct TagsQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/common/type_of.graphql",
  response_derives = "Debug"
)]
pub struct TypeOfQuery;

#[derive(Debug, Parser)]
pub struct Lookup {
  pub selector: Selector,
}

pub async fn lookup(context: &Context, data: Lookup) -> anyhow::Result<Uuid> {
  data.selector.resolve(context).await
}

#[derive(Debug, Parser)]
pub struct TypeOf {
  pub selector: Selector,
}

pub async fn type_of(context: &Context, data: TypeOf) -> anyhow::Result<EntityType> {
  let id = data.selector.resolve(context).await?;
  let response = context
    .request::<TypeOfQuery>(type_of_query::Variables { id })
    .await?;
  check_errors(&response.errors)?;
  let type_kind = match response.data.unwrap().type_of.as_str() {
    "user" => EntityType::User,
    "folder" => EntityType::Folder,
    "organization" => EntityType::Organization,
    "module" => EntityType::Module,
    "structure" => EntityType::Structure,
    "enumeration" => EntityType::Enumeration,
    _ => EntityType::Unknown,
  };
  Ok(type_kind)
}

#[derive(Debug)]
pub enum EntityType {
  User,
  Folder,
  Organization,
  Module,
  Structure,
  Enumeration,
  Unknown,
}

impl Display for EntityType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      EntityType::User => write!(f, "user"),
      EntityType::Folder => write!(f, "folder"),
      EntityType::Organization => write!(f, "organization"),
      EntityType::Module => write!(f, "module"),
      EntityType::Structure => write!(f, "structure"),
      EntityType::Enumeration => write!(f, "enumeration"),
      EntityType::Unknown => write!(f, "unknown"),
    }
  }
}

#[derive(Debug, Parser)]
pub struct Tags {
  pub selector: Selector,
}

pub async fn tags(context: &Context, data: Tags) -> anyhow::Result<Vec<Version>> {
  let id = data.selector.resolve(context).await?;
  let response = context
    .request::<TagsQuery>(tags_query::Variables { id })
    .await?;

  check_errors(&response.errors)?;

  let data = response.data.unwrap();

  let mut res = Vec::with_capacity(data.tags.len());
  for tag in data.tags {
    res.push(tag);
  }

  Ok(res)
}
