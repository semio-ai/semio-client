use crate::common::*;
use crate::context::Context;
use clap::Parser;
use graphql_client::GraphQLQuery;
use semio_record::acl::Acl;
use semio_record::record::{RecordDefn, Version, VersionReq};
use semio_record::structure::v0::{frozen, unfrozen, Structure};
use semio_record::ty::{FrozenTy, UnfrozenTy};
use indexmap::IndexMap;
use uuid::Uuid;
use uuid::Uuid as UUID;

macro_rules! impl_unfrozen_structure_field_from_response {
  ($query: path) => {
    impl FromResponse<$query> for unfrozen::StructureField {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          name: value.name,
          ty: UnfrozenTy::from_response(value.type_)?,
        })
      }
    }
  };
}

macro_rules! impl_frozen_structure_field_from_response {
  ($query: path) => {
    impl FromResponse<$query> for frozen::StructureField {
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
    impl FromResponse<$query> for <Structure as RecordDefn>::Public {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut fields = IndexMap::new();
        for id_field in value.fields {
          fields.insert(
            id_field.id,
            unfrozen::StructureField::from_response(id_field.field)?,
          );
        }
        Ok(Self {
          parent: value.parent,
          name: value.name,
          fields,
        })
      }
    }
  };
}

macro_rules! impl_private_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Structure as RecordDefn>::Private {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut fields = IndexMap::new();
        for id_field in value.fields {
          fields.insert(
            id_field.id,
            unfrozen::StructureField::from_response(id_field.field)?,
          );
        }
        Ok(Self {
          acl: Acl::from_response(value.acl)?,
          parent: value.parent,
          name: value.name,
          fields,
        })
      }
    }
  };
}

macro_rules! impl_frozen_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Structure as RecordDefn>::Frozen {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut fields = IndexMap::new();
        for id_field in value.fields {
          fields.insert(
            id_field.id,
            frozen::StructureField::from_response(id_field.field)?,
          );
        }
        Ok(Self {
          parent: value.parent,
          name: value.name,
          fields,
        })
      }
    }
  };
}

impl_primitive_kind_from_response!(get_public_query::PrimitiveKind);
impl_primitive_from_response!(
  get_public_query::GetPublicQueryStructureGetPublicFieldsFieldTypeOnPrimitive
);
impl_unfrozen_ty_from_response!(get_public_query::GetPublicQueryStructureGetPublicFieldsFieldType);
impl_unfrozen_structure_field_from_response!(
  get_public_query::GetPublicQueryStructureGetPublicFieldsField
);
impl_public_from_response!(get_public_query::GetPublicQueryStructureGetPublic);

impl_permission_level_from_response!(get_private_query::PermissionLevel);
impl_with_permissions_from_response!(
  get_private_query::GetPrivateQueryStructureGetPrivateAclDefault
);
impl_with_permissions_from_response!(
  get_private_query::GetPrivateQueryStructureGetPrivateAclPermissionsWithPermissions
);
impl_acl_from_response!(get_private_query::GetPrivateQueryStructureGetPrivateAcl);
impl_primitive_kind_from_response!(get_private_query::PrimitiveKind);
impl_primitive_from_response!(
  get_private_query::GetPrivateQueryStructureGetPrivateFieldsFieldTypeOnPrimitive
);
impl_unfrozen_ty_from_response!(
  get_private_query::GetPrivateQueryStructureGetPrivateFieldsFieldType
);
impl_unfrozen_structure_field_from_response!(
  get_private_query::GetPrivateQueryStructureGetPrivateFieldsField
);
impl_private_from_response!(get_private_query::GetPrivateQueryStructureGetPrivate);

impl_primitive_kind_from_response!(get_version_public_query::PrimitiveKind);
impl_primitive_from_response!(get_version_public_query::GetVersionPublicQueryStructureGetVersionPublicFieldsFieldTypeOnPrimitive);
impl_unfrozen_ty_from_response!(
  get_version_public_query::GetVersionPublicQueryStructureGetVersionPublicFieldsFieldType
);
impl_unfrozen_structure_field_from_response!(
  get_version_public_query::GetVersionPublicQueryStructureGetVersionPublicFieldsField
);
impl_public_from_response!(
  get_version_public_query::GetVersionPublicQueryStructureGetVersionPublic
);

impl_permission_level_from_response!(get_version_private_query::PermissionLevel);
impl_with_permissions_from_response!(
  get_version_private_query::GetVersionPrivateQueryStructureGetVersionPrivateAclDefault
);
impl_with_permissions_from_response!(get_version_private_query::GetVersionPrivateQueryStructureGetVersionPrivateAclPermissionsWithPermissions);
impl_acl_from_response!(
  get_version_private_query::GetVersionPrivateQueryStructureGetVersionPrivateAcl
);
impl_primitive_kind_from_response!(get_version_private_query::PrimitiveKind);
impl_primitive_from_response!(get_version_private_query::GetVersionPrivateQueryStructureGetVersionPrivateFieldsFieldTypeOnPrimitive);
impl_unfrozen_ty_from_response!(
  get_version_private_query::GetVersionPrivateQueryStructureGetVersionPrivateFieldsFieldType
);
impl_unfrozen_structure_field_from_response!(
  get_version_private_query::GetVersionPrivateQueryStructureGetVersionPrivateFieldsField
);
impl_private_from_response!(
  get_version_private_query::GetVersionPrivateQueryStructureGetVersionPrivate
);

// tagged_query
impl_primitive_kind_from_response!(tagged_query::PrimitiveKind);
impl_primitive_from_response!(tagged_query::TaggedQueryStructureTaggedFieldsFieldTypeOnPrimitive);
impl_frozen_ty_from_response!(tagged_query::TaggedQueryStructureTaggedFieldsFieldType);
impl_frozen_structure_field_from_response!(tagged_query::TaggedQueryStructureTaggedFieldsField);
impl_frozen_from_response!(tagged_query::TaggedQueryStructureTagged);

// tagged_req_query
impl_primitive_kind_from_response!(tagged_req_query::PrimitiveKind);
impl_primitive_from_response!(
  tagged_req_query::TaggedReqQueryStructureTaggedReqFieldsFieldTypeOnPrimitive
);
impl_frozen_ty_from_response!(tagged_req_query::TaggedReqQueryStructureTaggedReqFieldsFieldType);
impl_frozen_structure_field_from_response!(
  tagged_req_query::TaggedReqQueryStructureTaggedReqFieldsField
);
impl_frozen_from_response!(tagged_req_query::TaggedReqQueryStructureTaggedReq);

// get_frozen_query
impl_primitive_kind_from_response!(get_frozen_query::PrimitiveKind);
impl_primitive_from_response!(get_frozen_query::GetFrozenQueryStructureGetFrozenFieldsFieldTypeOnPrimitive);
impl_frozen_ty_from_response!(get_frozen_query::GetFrozenQueryStructureGetFrozenFieldsFieldType);
impl_frozen_structure_field_from_response!(get_frozen_query::GetFrozenQueryStructureGetFrozenFieldsField);
impl_frozen_from_response!(get_frozen_query::GetFrozenQueryStructureGetFrozen);

// get_version_frozen_query
impl_primitive_kind_from_response!(get_version_frozen_query::PrimitiveKind);
impl_primitive_from_response!(get_version_frozen_query::GetVersionFrozenQueryStructureGetVersionFrozenFieldsFieldTypeOnPrimitive);
impl_frozen_ty_from_response!(get_version_frozen_query::GetVersionFrozenQueryStructureGetVersionFrozenFieldsFieldType);
impl_frozen_structure_field_from_response!(get_version_frozen_query::GetVersionFrozenQueryStructureGetVersionFrozenFieldsField);
impl_frozen_from_response!(get_version_frozen_query::GetVersionFrozenQueryStructureGetVersionFrozen);

impl_create!(structure, "src/structure/create.graphql");
impl_get_public!(
  <Structure as RecordDefn>::Public,
  structure,
  "src/structure/get_public.graphql"
);
impl_get_private!(
  <Structure as RecordDefn>::Private,
  structure,
  "src/structure/get_private.graphql"
);
impl_get_version_public!(
  <Structure as RecordDefn>::Public,
  structure,
  "src/structure/get_version_public.graphql"
);
impl_get_version_private!(
  <Structure as RecordDefn>::Private,
  structure,
  "src/structure/get_version_private.graphql"
);
impl_set_name!(structure, "src/structure/set_name.graphql");
impl_set_parent!(structure, "src/structure/set_parent.graphql");
impl_add_permissions!(structure, "src/structure/add_permissions.graphql");
impl_set_permissions!(structure, "src/structure/set_permissions.graphql");
impl_remove_permissions!(structure, "src/structure/remove_permissions.graphql");
impl_set_default_permissions!(structure, "src/structure/set_default_permissions.graphql");
impl_tag!("src/structure/tag.graphql");
impl_tagged!(
  <Structure as RecordDefn>::Frozen,
  structure,
  "src/structure/tagged.graphql"
);
impl_tagged_req!(
  <Structure as RecordDefn>::Frozen,
  structure,
  "src/structure/tagged_req.graphql"
);

impl_get_frozen!(
  <Structure as RecordDefn>::Frozen,
  structure,
  "src/structure/get_frozen.graphql"
);

impl_get_version_frozen!(
  <Structure as RecordDefn>::Frozen,
  structure,
  "src/structure/get_version_frozen.graphql"
);

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/structure/add_field.graphql",
  response_derives = "Debug"
)]
pub struct AddFieldQuery;

#[derive(Debug, Parser)]
pub struct AddField {
  pub selector: Selector,
  pub name: String,
  pub ty: UnfrozenSelectorTy,
}

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/structure/remove_field.graphql",
  response_derives = "Debug"
)]
pub struct RemoveFieldQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/structure/set_field_type.graphql",
  response_derives = "Debug"
)]
pub struct SetFieldTypeQuery;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/structure/set_field_name.graphql",
  response_derives = "Debug"
)]
pub struct SetFieldNameQuery;

#[derive(Debug, Parser)]
pub struct RemoveField {
  pub selector: Selector,
  pub id: Uuid,
}

#[derive(Debug, Parser)]
pub struct SetFieldType {
  pub selector: Selector,
  pub id: Uuid,
  pub ty: UnfrozenSelectorTy,
}

#[derive(Debug, Parser)]
pub struct SetFieldName {
  pub selector: Selector,
  pub id: Uuid,
  pub name: String,
}

pub async fn add_field<'a>(context: &Context, data: AddField) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let ty = data.ty.resolve(context).await?;
  let response = context
    .request::<AddFieldQuery>(add_field_query::Variables {
      id,
      name: data.name,
      ty: ty.to_string(),
    })
    .await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().structure.add_field)
}

pub async fn remove_field<'a>(context: &Context, data: RemoveField) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let response = context
    .request::<RemoveFieldQuery>(remove_field_query::Variables {
      id,
      field_id: data.id,
    })
    .await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().structure.remove_field)
}

pub async fn set_field_type<'a>(context: &Context, data: SetFieldType) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let ty = data.ty.resolve(context).await?;
  let response = context
    .request::<SetFieldTypeQuery>(set_field_type_query::Variables {
      id,
      field_id: data.id,
      ty: ty.to_string(),
    })
    .await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().structure.set_field_type)
}

pub async fn set_field_name<'a>(context: &Context, data: SetFieldName) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let response = context
    .request::<SetFieldNameQuery>(set_field_name_query::Variables {
      id,
      field_id: data.id,
      name: data.name,
    })
    .await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().structure.set_field_name)
}
