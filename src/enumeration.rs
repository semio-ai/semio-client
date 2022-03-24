use crate::common::*;
use crate::context::Context;
use clap::Parser;
use semio_record::acl::Acl;
use semio_record::enumeration::v0::{frozen, unfrozen, Enumeration};
use semio_record::record::{RecordDefn, Version, VersionReq};
use semio_record::ty::{FrozenTy, UnfrozenTy};
use std::collections::HashMap;
use uuid::Uuid;

macro_rules! impl_unfrozen_enumeration_variant_from_response {
  ($query: path) => {
    impl FromResponse<$query> for unfrozen::EnumerationVariant {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          name: value.name,
          ty: UnfrozenTy::from_response(value.type_)?,
        })
      }
    }
  };
}

macro_rules! impl_frozen_enumeration_variant_from_response {
  ($query: path) => {
    impl FromResponse<$query> for frozen::EnumerationVariant {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          name: value.name,
          ty: FrozenTy::from_response(value.type_)?,
        })
      }
    }
  };
}

macro_rules! impl_public_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Enumeration as RecordDefn>::Public {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut variants = HashMap::new();
        for id_variant in value.variants {
          variants.insert(
            id_variant.id,
            unfrozen::EnumerationVariant::from_response(id_variant.variant)?,
          );
        }
        Ok(Self {
          parent: value.parent,
          name: value.name,
          variants,
        })
      }
    }
  };
}

macro_rules! impl_private_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Enumeration as RecordDefn>::Private {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut variants = HashMap::new();
        for id_variant in value.variants {
          variants.insert(
            id_variant.id,
            unfrozen::EnumerationVariant::from_response(id_variant.variant)?,
          );
        }
        Ok(Self {
          acl: Acl::from_response(value.acl)?,
          parent: value.parent,
          name: value.name,
          variants,
        })
      }
    }
  };
}

macro_rules! impl_frozen_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Enumeration as RecordDefn>::Frozen {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut variants = HashMap::new();
        for id_variant in value.variants {
          variants.insert(
            id_variant.id,
            frozen::EnumerationVariant::from_response(id_variant.variant)?,
          );
        }
        Ok(Self {
          parent: value.parent,
          name: value.name,
          variants,
        })
      }
    }
  };
}

// get_public_query
impl_primitive_kind_from_response!(get_public_query::PrimitiveKind);
impl_primitive_from_response!(
  get_public_query::GetPublicQueryEnumerationGetPublicVariantsVariantTypeOnPrimitive
);
impl_unfrozen_ty_from_response!(
  get_public_query::GetPublicQueryEnumerationGetPublicVariantsVariantType
);
impl_unfrozen_enumeration_variant_from_response!(
  get_public_query::GetPublicQueryEnumerationGetPublicVariantsVariant
);
impl_public_from_response!(get_public_query::GetPublicQueryEnumerationGetPublic);

// get_private_query
impl_permission_level_from_response!(get_private_query::PermissionLevel);
impl_with_permissions_from_response!(
  get_private_query::GetPrivateQueryEnumerationGetPrivateAclDefault
);
impl_with_permissions_from_response!(
  get_private_query::GetPrivateQueryEnumerationGetPrivateAclPermissionsWithPermissions
);
impl_acl_from_response!(get_private_query::GetPrivateQueryEnumerationGetPrivateAcl);
impl_primitive_kind_from_response!(get_private_query::PrimitiveKind);
impl_primitive_from_response!(
  get_private_query::GetPrivateQueryEnumerationGetPrivateVariantsVariantTypeOnPrimitive
);
impl_unfrozen_ty_from_response!(
  get_private_query::GetPrivateQueryEnumerationGetPrivateVariantsVariantType
);
impl_unfrozen_enumeration_variant_from_response!(
  get_private_query::GetPrivateQueryEnumerationGetPrivateVariantsVariant
);
impl_private_from_response!(get_private_query::GetPrivateQueryEnumerationGetPrivate);

// get_version_public_query
impl_primitive_kind_from_response!(get_version_public_query::PrimitiveKind);
impl_primitive_from_response!(get_version_public_query::GetVersionPublicQueryEnumerationGetVersionPublicVariantsVariantTypeOnPrimitive);
impl_unfrozen_ty_from_response!(
  get_version_public_query::GetVersionPublicQueryEnumerationGetVersionPublicVariantsVariantType
);
impl_unfrozen_enumeration_variant_from_response!(
  get_version_public_query::GetVersionPublicQueryEnumerationGetVersionPublicVariantsVariant
);
impl_public_from_response!(
  get_version_public_query::GetVersionPublicQueryEnumerationGetVersionPublic
);

// get_version_private_query
impl_permission_level_from_response!(get_version_private_query::PermissionLevel);
impl_with_permissions_from_response!(
  get_version_private_query::GetVersionPrivateQueryEnumerationGetVersionPrivateAclDefault
);
impl_with_permissions_from_response!(get_version_private_query::GetVersionPrivateQueryEnumerationGetVersionPrivateAclPermissionsWithPermissions);
impl_acl_from_response!(
  get_version_private_query::GetVersionPrivateQueryEnumerationGetVersionPrivateAcl
);
impl_primitive_kind_from_response!(get_version_private_query::PrimitiveKind);
impl_primitive_from_response!(get_version_private_query::GetVersionPrivateQueryEnumerationGetVersionPrivateVariantsVariantTypeOnPrimitive);
impl_unfrozen_ty_from_response!(
  get_version_private_query::GetVersionPrivateQueryEnumerationGetVersionPrivateVariantsVariantType
);
impl_unfrozen_enumeration_variant_from_response!(
  get_version_private_query::GetVersionPrivateQueryEnumerationGetVersionPrivateVariantsVariant
);
impl_private_from_response!(
  get_version_private_query::GetVersionPrivateQueryEnumerationGetVersionPrivate
);

// tagged_query
impl_primitive_kind_from_response!(tagged_query::PrimitiveKind);
impl_primitive_from_response!(
  tagged_query::TaggedQueryEnumerationTaggedVariantsVariantTypeOnPrimitive
);
impl_frozen_ty_from_response!(tagged_query::TaggedQueryEnumerationTaggedVariantsVariantType);
impl_frozen_enumeration_variant_from_response!(
  tagged_query::TaggedQueryEnumerationTaggedVariantsVariant
);
impl_frozen_from_response!(tagged_query::TaggedQueryEnumerationTagged);

// tagged_req_query
impl_primitive_kind_from_response!(tagged_req_query::PrimitiveKind);
impl_primitive_from_response!(
  tagged_req_query::TaggedReqQueryEnumerationTaggedReqVariantsVariantTypeOnPrimitive
);
impl_frozen_ty_from_response!(
  tagged_req_query::TaggedReqQueryEnumerationTaggedReqVariantsVariantType
);
impl_frozen_enumeration_variant_from_response!(
  tagged_req_query::TaggedReqQueryEnumerationTaggedReqVariantsVariant
);
impl_frozen_from_response!(tagged_req_query::TaggedReqQueryEnumerationTaggedReq);

impl_create!(enumeration, "src/enumeration/create.graphql");
impl_get_public!(
  <Enumeration as RecordDefn>::Public,
  enumeration,
  "src/enumeration/get_public.graphql"
);
impl_get_private!(
  <Enumeration as RecordDefn>::Private,
  enumeration,
  "src/enumeration/get_private.graphql"
);
impl_get_version_public!(
  <Enumeration as RecordDefn>::Public,
  enumeration,
  "src/enumeration/get_version_public.graphql"
);
impl_get_version_private!(
  <Enumeration as RecordDefn>::Private,
  enumeration,
  "src/enumeration/get_version_private.graphql"
);
impl_set_name!(enumeration, "src/enumeration/set_name.graphql");
impl_set_parent!(enumeration, "src/enumeration/set_parent.graphql");
impl_add_permissions!(enumeration, "src/enumeration/add_permissions.graphql");
impl_set_permissions!(enumeration, "src/enumeration/set_permissions.graphql");
impl_remove_permissions!(enumeration, "src/enumeration/remove_permissions.graphql");
impl_set_default_permissions!(
  enumeration,
  "src/enumeration/set_default_permissions.graphql"
);
impl_tag!("src/enumeration/tag.graphql");
impl_tagged!(
  <Enumeration as RecordDefn>::Frozen,
  enumeration,
  "src/enumeration/tagged.graphql"
);
impl_tagged_req!(
  <Enumeration as RecordDefn>::Frozen,
  enumeration,
  "src/enumeration/tagged_req.graphql"
);

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/enumeration/add_variant.graphql",
  response_derives = "Debug"
)]
pub struct AddVariantQuery;

#[derive(Debug, Parser)]
pub struct AddVariant {
  pub selector: Selector,
  pub name: String,
  pub ty: UnfrozenSelectorTy,
}

pub async fn add_variant<'a>(context: &Context, data: AddVariant) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let ty = data.ty.resolve(context).await?;
  let response = context
    .request::<AddVariantQuery>(add_variant_query::Variables {
      id,
      name: data.name,
      ty: ty.to_string(),
    })
    .await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().enumeration.add_variant)
}

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/enumeration/remove_variant.graphql",
  response_derives = "Debug"
)]
pub struct RemoveVariantQuery;

#[derive(Debug, Parser)]
pub struct RemoveVariant {
  pub selector: Selector,
  pub id: Uuid,
}

pub async fn remove_variant<'a>(context: &Context, data: RemoveVariant) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let response = context
    .request::<RemoveVariantQuery>(remove_variant_query::Variables {
      id,
      variant_id: data.id,
    })
    .await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().enumeration.remove_variant)
}

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/enumeration/set_variant_type.graphql",
  response_derives = "Debug"
)]
pub struct SetVariantTypeQuery;

#[derive(Debug, Parser)]
pub struct SetVariantType {
  pub selector: Selector,
  pub id: Uuid,
  pub ty: UnfrozenSelectorTy,
}

pub async fn set_variant_type<'a>(context: &Context, data: SetVariantType) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let ty = data.ty.resolve(context).await?;

  let response = context
    .request::<SetVariantTypeQuery>(set_variant_type_query::Variables {
      id,
      variant_id: data.id,
      ty: ty.to_string(),
    })
    .await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().enumeration.set_variant_type)
}

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/enumeration/set_variant_name.graphql",
  response_derives = "Debug"
)]
pub struct SetVariantNameQuery;

#[derive(Debug, Parser)]
pub struct SetVariantName {
  pub selector: Selector,
  pub id: Uuid,
  pub name: String,
}

pub async fn set_variant_name<'a>(context: &Context, data: SetVariantName) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let response = context
    .request::<SetVariantNameQuery>(set_variant_name_query::Variables {
      id,
      variant_id: data.id,
      name: data.name,
    })
    .await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().enumeration.set_variant_name)
}
