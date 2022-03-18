use std::collections::HashMap;

use graphql_client::GraphQLQuery;
use clap::Parser;
use semio_record::module::v0::Module;
use semio_record::module::v0::{unfrozen, frozen};

use semio_record::ty::{UnfrozenTy, FrozenTy};
use semio_record::record::{UnfrozenReference, FrozenReference};
use semio_record::acl::Acl;
use uuid::Uuid;

use crate::{context::Context};

use crate::common::*;

use semio_record::record::{Version, VersionReq, RecordDefn};


// Get public query
impl_primitive_kind_from_response!(get_public_query::PrimitiveKind);
impl_primitive_from_response!(get_public_query::GetPublicQueryModuleGetPublicExportsExportKindOnFunctionReturnTypeOnPrimitive);
impl_unfrozen_ty_from_response!(get_public_query::GetPublicQueryModuleGetPublicExportsExportKindOnFunctionReturnType);
impl_primitive_from_response!(get_public_query::GetPublicQueryModuleGetPublicExportsExportKindOnFunctionParametersParameterTypeOnPrimitive);
impl_unfrozen_ty_from_response!(get_public_query::GetPublicQueryModuleGetPublicExportsExportKindOnFunctionParametersParameterType);

// get_version_public_query
impl_primitive_kind_from_response!(get_version_public_query::PrimitiveKind);
impl_primitive_from_response!(get_version_public_query::GetVersionPublicQueryModuleGetVersionPublicExportsExportKindOnFunctionReturnTypeOnPrimitive);
impl_unfrozen_ty_from_response!(get_version_public_query::GetVersionPublicQueryModuleGetVersionPublicExportsExportKindOnFunctionReturnType);
impl_primitive_from_response!(get_version_public_query::GetVersionPublicQueryModuleGetVersionPublicExportsExportKindOnFunctionParametersParameterTypeOnPrimitive);
impl_unfrozen_ty_from_response!(get_version_public_query::GetVersionPublicQueryModuleGetVersionPublicExportsExportKindOnFunctionParametersParameterType);

// get_private_query
impl_primitive_kind_from_response!(get_private_query::PrimitiveKind);
impl_primitive_from_response!(get_private_query::GetPrivateQueryModuleGetPrivateExportsExportKindOnFunctionReturnTypeOnPrimitive);
impl_unfrozen_ty_from_response!(get_private_query::GetPrivateQueryModuleGetPrivateExportsExportKindOnFunctionReturnType);
impl_primitive_from_response!(get_private_query::GetPrivateQueryModuleGetPrivateExportsExportKindOnFunctionParametersParameterTypeOnPrimitive);
impl_unfrozen_ty_from_response!(get_private_query::GetPrivateQueryModuleGetPrivateExportsExportKindOnFunctionParametersParameterType);

// get_version_private_query
impl_primitive_kind_from_response!(get_version_private_query::PrimitiveKind);
impl_primitive_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivateExportsExportKindOnFunctionReturnTypeOnPrimitive);
impl_unfrozen_ty_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivateExportsExportKindOnFunctionReturnType);
impl_primitive_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivateExportsExportKindOnFunctionParametersParameterTypeOnPrimitive);
impl_unfrozen_ty_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivateExportsExportKindOnFunctionParametersParameterType);

macro_rules! impl_unfrozen_parameter_from_response {
  ($query: path) => {
    impl FromResponse<$query> for unfrozen::Parameter {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          name: value.name,
          ty: UnfrozenTy::from_response(value.type_)?,
          mutable: value.mutable,
        })
      }
    }
  }
}

macro_rules! impl_unfrozen_function_from_response {
  ($query: path) => {
    impl FromResponse<$query> for unfrozen::Function {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut parameters = HashMap::new();
        let mut parameter_ordering = Vec::with_capacity(value.parameters.len());
        for id_parameter in value.parameters {
          parameter_ordering.push(id_parameter.id.clone());
          parameters.insert(id_parameter.id, unfrozen::Parameter::from_response(id_parameter.parameter)?);
        }
        
        Ok(Self {
          parameters,
          parameter_ordering,
          return_ty: UnfrozenTy::from_response(value.return_type)?,
        })
      }
    }
  }
}

macro_rules! impl_unfrozen_export_kind_from_response {
  ($query: path) => {
    impl FromResponse<$query> for unfrozen::ExportKind {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        match value {
          <$query>::Function(value) => Ok(Self::Function(unfrozen::Function::from_response(value)?)),
          _ => Err(anyhow::anyhow!("unexpected export kind {:?}", value)),
        }
      }
    }
  }
}

macro_rules! impl_unfrozen_export_from_response {
  ($query: path) => {
    impl FromResponse<$query> for unfrozen::Export {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          name: value.name,
          kind: unfrozen::ExportKind::from_response(value.kind)?,
        })
      }
    }
  }
}

macro_rules! impl_frozen_parameter_from_response {
  ($query: path) => {
    impl FromResponse<$query> for frozen::Parameter {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          name: value.name,
          ty: FrozenTy::from_response(value.type_)?,
          mutable: value.mutable,
        })
      }
    }
  }
}

macro_rules! impl_frozen_function_from_response {
  ($query: path) => {
    impl FromResponse<$query> for frozen::Function {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut parameters = HashMap::new();
        let mut parameter_ordering = Vec::with_capacity(value.parameters.len());
        for id_parameter in value.parameters {
          parameter_ordering.push(id_parameter.id.clone());
          parameters.insert(id_parameter.id, frozen::Parameter::from_response(id_parameter.parameter)?);
        }
        
        Ok(Self {
          parameters,
          parameter_ordering,
          return_ty: FrozenTy::from_response(value.return_type)?,
        })
      }
    }
  }
}

macro_rules! impl_frozen_export_kind_from_response {
  ($query: path) => {
    impl FromResponse<$query> for frozen::ExportKind {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        match value {
          <$query>::FrozenFunction(value) => Ok(Self::Function(frozen::Function::from_response(value)?)),
          _ => Err(anyhow::anyhow!("unexpected export kind {:?}", value)),
        }
      }
    }
  }
}

macro_rules! impl_frozen_export_from_response {
  ($query: path) => {
    impl FromResponse<$query> for frozen::Export {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          name: value.name,
          kind: frozen::ExportKind::from_response(value.kind)?,
        })
      }
    }
  }
}

macro_rules! impl_public_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Module as RecordDefn>::Public {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut exports = HashMap::new();
        for id_export in value.exports {
          exports.insert(id_export.id, unfrozen::Export::from_response(id_export.export)?);
        }

        let mut dependencies = Vec::with_capacity(value.dependencies.len());
        for dependency in value.dependencies {
          dependencies.push(UnfrozenReference {
            id: dependency.id,
            version_req: dependency.version_req,
          });
        }
        
        Ok(Self {
          parent: value.parent,
          name: value.name,
          executable: value.executable,
          exports,
          dependencies,
        })
      }
    }
  }
}

macro_rules! impl_private_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Module as RecordDefn>::Private {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut exports = HashMap::new();
        for id_export in value.exports {
          exports.insert(id_export.id, unfrozen::Export::from_response(id_export.export)?);
        }

        let mut dependencies = Vec::with_capacity(value.dependencies.len());
        for dependency in value.dependencies {
          dependencies.push(UnfrozenReference {
            id: dependency.id,
            version_req: dependency.version_req,
          });
        }
        
        Ok(Self {
          acl: Acl::from_response(value.acl)?,
          parent: value.parent,
          name: value.name,
          executable: value.executable,
          exports,
          dependencies,
        })
      }
    }
  }
}

macro_rules! impl_frozen_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Module as RecordDefn>::Frozen {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        let mut exports = HashMap::new();
        for id_export in value.exports {
          exports.insert(id_export.id, frozen::Export::from_response(id_export.export)?);
        }

        let mut dependencies = Vec::with_capacity(value.dependencies.len());
        for dependency in value.dependencies {
          dependencies.push(FrozenReference {
            id: dependency.id,
            version: dependency.version,
          });
        }
        
        Ok(Self {
          parent: value.parent.ok_or_else(|| anyhow::anyhow!("missing parent"))?,
          name: value.name,
          executable: value.executable,
          exports,
          dependencies,
        })
      }
    }
  }
}

// get_public_query
impl_unfrozen_parameter_from_response!(get_public_query::GetPublicQueryModuleGetPublicExportsExportKindOnFunctionParametersParameter);
impl_unfrozen_function_from_response!(get_public_query::GetPublicQueryModuleGetPublicExportsExportKindOnFunction);
impl_unfrozen_export_kind_from_response!(get_public_query::GetPublicQueryModuleGetPublicExportsExportKind);
impl_unfrozen_export_from_response!(get_public_query::GetPublicQueryModuleGetPublicExportsExport);
impl_public_from_response!(get_public_query::GetPublicQueryModuleGetPublic);

// get_version_public_query
impl_unfrozen_parameter_from_response!(get_version_public_query::GetVersionPublicQueryModuleGetVersionPublicExportsExportKindOnFunctionParametersParameter);
impl_unfrozen_function_from_response!(get_version_public_query::GetVersionPublicQueryModuleGetVersionPublicExportsExportKindOnFunction);
impl_unfrozen_export_kind_from_response!(get_version_public_query::GetVersionPublicQueryModuleGetVersionPublicExportsExportKind);
impl_unfrozen_export_from_response!(get_version_public_query::GetVersionPublicQueryModuleGetVersionPublicExportsExport);
impl_public_from_response!(get_version_public_query::GetVersionPublicQueryModuleGetVersionPublic);

// get_private_query
impl_unfrozen_parameter_from_response!(get_private_query::GetPrivateQueryModuleGetPrivateExportsExportKindOnFunctionParametersParameter);
impl_unfrozen_function_from_response!(get_private_query::GetPrivateQueryModuleGetPrivateExportsExportKindOnFunction);
impl_unfrozen_export_kind_from_response!(get_private_query::GetPrivateQueryModuleGetPrivateExportsExportKind);
impl_unfrozen_export_from_response!(get_private_query::GetPrivateQueryModuleGetPrivateExportsExport);
impl_private_from_response!(get_private_query::GetPrivateQueryModuleGetPrivate);
impl_with_permissions_from_response!(get_private_query::GetPrivateQueryModuleGetPrivateAclPermissionsWithPermissions);
impl_with_permissions_from_response!(get_private_query::GetPrivateQueryModuleGetPrivateAclDefault);
impl_acl_from_response!(get_private_query::GetPrivateQueryModuleGetPrivateAcl);
impl_permission_level_from_response!(get_private_query::PermissionLevel);

// get_version_private_query
impl_unfrozen_parameter_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivateExportsExportKindOnFunctionParametersParameter);
impl_unfrozen_function_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivateExportsExportKindOnFunction);
impl_unfrozen_export_kind_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivateExportsExportKind);
impl_unfrozen_export_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivateExportsExport);
impl_private_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivate);
impl_with_permissions_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivateAclPermissionsWithPermissions);
impl_with_permissions_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivateAclDefault);
impl_acl_from_response!(get_version_private_query::GetVersionPrivateQueryModuleGetVersionPrivateAcl);
impl_permission_level_from_response!(get_version_private_query::PermissionLevel);


// tagged_req_query
impl_primitive_kind_from_response!(tagged_req_query::PrimitiveKind);
impl_primitive_from_response!(tagged_req_query::TaggedReqQueryModuleTaggedReqExportsExportKindOnFrozenFunctionParametersParameterTypeOnPrimitive);
impl_primitive_from_response!(tagged_req_query::TaggedReqQueryModuleTaggedReqExportsExportKindOnFrozenFunctionReturnTypeOnPrimitive);
impl_frozen_ty_from_response!(tagged_req_query::TaggedReqQueryModuleTaggedReqExportsExportKindOnFrozenFunctionParametersParameterType);
impl_frozen_ty_from_response!(tagged_req_query::TaggedReqQueryModuleTaggedReqExportsExportKindOnFrozenFunctionReturnType);
impl_frozen_parameter_from_response!(tagged_req_query::TaggedReqQueryModuleTaggedReqExportsExportKindOnFrozenFunctionParametersParameter);
impl_frozen_function_from_response!(tagged_req_query::TaggedReqQueryModuleTaggedReqExportsExportKindOnFrozenFunction);
impl_frozen_export_kind_from_response!(tagged_req_query::TaggedReqQueryModuleTaggedReqExportsExportKind);
impl_frozen_export_from_response!(tagged_req_query::TaggedReqQueryModuleTaggedReqExportsExport);
impl_frozen_from_response!(tagged_req_query::TaggedReqQueryModuleTaggedReq);

// tagged_query
impl_primitive_kind_from_response!(tagged_query::PrimitiveKind);
impl_primitive_from_response!(tagged_query::TaggedQueryModuleTaggedExportsExportKindOnFrozenFunctionParametersParameterTypeOnPrimitive);
impl_primitive_from_response!(tagged_query::TaggedQueryModuleTaggedExportsExportKindOnFrozenFunctionReturnTypeOnPrimitive);
impl_frozen_ty_from_response!(tagged_query::TaggedQueryModuleTaggedExportsExportKindOnFrozenFunctionParametersParameterType);
impl_frozen_ty_from_response!(tagged_query::TaggedQueryModuleTaggedExportsExportKindOnFrozenFunctionReturnType);
impl_frozen_parameter_from_response!(tagged_query::TaggedQueryModuleTaggedExportsExportKindOnFrozenFunctionParametersParameter);
impl_frozen_function_from_response!(tagged_query::TaggedQueryModuleTaggedExportsExportKindOnFrozenFunction);
impl_frozen_export_kind_from_response!(tagged_query::TaggedQueryModuleTaggedExportsExportKind);
impl_frozen_export_from_response!(tagged_query::TaggedQueryModuleTaggedExportsExport);
impl_frozen_from_response!(tagged_query::TaggedQueryModuleTagged);



impl_create!(module, "src/module/create.graphql");
impl_get_public!(<Module as RecordDefn>::Public, module, "src/module/get_public.graphql");
impl_get_private!(<Module as RecordDefn>::Private, module, "src/module/get_private.graphql");
impl_get_version_public!(<Module as RecordDefn>::Public, module, "src/module/get_version_public.graphql");
impl_get_version_private!(<Module as RecordDefn>::Private, module, "src/module/get_version_private.graphql");
impl_set_name!(module, "src/module/set_name.graphql");
impl_set_parent!(module, "src/module/set_parent.graphql");
impl_add_permissions!(module, "src/module/add_permissions.graphql");
impl_set_permissions!(module, "src/module/set_permissions.graphql");
impl_remove_permissions!(module, "src/module/remove_permissions.graphql");
impl_set_default_permissions!(module, "src/module/set_default_permissions.graphql");
impl_tag!("src/module/tag.graphql");
impl_tagged!(<Module as RecordDefn>::Frozen, module, "src/module/tagged.graphql");
impl_tagged_req!(<Module as RecordDefn>::Frozen, module, "src/module/tagged_req.graphql");

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/module/set_executable.graphql",
  response_derives = "Debug"
)]
pub struct SetExecutableQuery;

#[derive(Debug, Parser)]
pub struct SetExecutable {
  pub selector: Selector,
  pub executable: Uuid,
}

async fn set_executable<'a>(context: &Context, data: SetExecutable) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let response = context.request::<SetExecutableQuery>(set_executable_query::Variables {
    id,
    blob_id: data.executable
  }).await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().module.set_executable)
}

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/module/add_function.graphql",
  response_derives = "Debug"
)]
pub struct AddFunctionQuery;

#[derive(Debug, Parser)]
pub struct AddFunction {
  pub selector: Selector,
  pub name: String,
  pub return_type: UnfrozenSelectorTy,
}

async fn add_function<'a>(context: &Context, data: AddFunction) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let return_type = data.return_type.resolve(context).await?;
  let response = context.request::<AddFunctionQuery>(add_function_query::Variables {
    id,
    name: data.name,
    returns: return_type.to_string()
  }).await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().module.add_function)
}

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/module/append_function_parameter.graphql",
  response_derives = "Debug"
)]
pub struct AppendFunctionParameterQuery;

#[derive(Debug, Parser)]
pub struct AppendFunctionParameter {
  pub selector: Selector,
  pub export_id: Uuid,
  pub name: String,
  pub ty: UnfrozenSelectorTy,
  #[clap(short, long)]
  pub mutable: Option<bool>,
}

async fn append_function_parameter<'a>(context: &Context, data: AppendFunctionParameter) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let ty = data.ty.resolve(context).await?;
  let response = context.request::<AppendFunctionParameterQuery>(append_function_parameter_query::Variables {
    id,
    export_id: data.export_id,
    name: data.name,
    ty: ty.to_string(),
    mutable: data.mutable
  }).await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().module.append_function_parameter)
}

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/module/remove_function_parameter.graphql",
  response_derives = "Debug"
)]
pub struct RemoveFunctionParameterQuery;

#[derive(Debug, Parser)]
pub struct RemoveFunctionParameter {
  pub selector: Selector,
  pub export_id: Uuid,
  pub parameter_id: Uuid,
}

async fn remove_function_parameter<'a>(context: &Context, data: RemoveFunctionParameter) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let response = context.request::<RemoveFunctionParameterQuery>(remove_function_parameter_query::Variables {
    id,
    export_id: data.export_id,
    parameter_id: data.parameter_id
  }).await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().module.remove_function_parameter)
}

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/module/set_function_parameter_name.graphql",
  response_derives = "Debug"
)]
pub struct SetFunctionParameterNameQuery;

#[derive(Debug, Parser)]
pub struct SetFunctionParameterName {
  pub selector: Selector,
  pub export_id: Uuid,
  pub parameter_id: Uuid,
  pub name: String,
}

pub async fn set_function_parameter_name<'a>(context: &Context, data: SetFunctionParameterName) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let response = context.request::<SetFunctionParameterNameQuery>(set_function_parameter_name_query::Variables {
    id,
    export_id: data.export_id,
    parameter_id: data.parameter_id,
    name: data.name
  }).await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().module.set_function_parameter_name)
}

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/module/set_function_parameter_type.graphql",
  response_derives = "Debug"
)]
pub struct SetFunctionParameterTypeQuery;

#[derive(Debug, Parser)]
pub struct SetFunctionParameterType {
  pub selector: Selector,
  pub export_id: Uuid,
  pub parameter_id: Uuid,
  pub ty: UnfrozenSelectorTy,
}

pub async fn set_function_parameter_type<'a>(context: &Context, data: SetFunctionParameterType) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let ty = data.ty.resolve(context).await?;
  let response = context.request::<SetFunctionParameterTypeQuery>(set_function_parameter_type_query::Variables {
    id,
    export_id: data.export_id,
    parameter_id: data.parameter_id,
    ty: ty.to_string()
  }).await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().module.set_function_parameter_type)
}

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
  schema_path = "src/schema.graphql.json",
  query_path = "src/module/set_function_return_type.graphql",
  response_derives = "Debug"
)]
pub struct SetFunctionReturnTypeQuery;

#[derive(Debug, Parser)]
pub struct SetFunctionReturnType {
  pub selector: Selector,
  pub export_id: Uuid,
  pub ty: UnfrozenSelectorTy,
}

async fn set_function_return_type<'a>(context: &Context, data: SetFunctionReturnType) -> anyhow::Result<i64> {
  let id = data.selector.resolve(context).await?;
  let ty = data.ty.resolve(context).await?;
  let response = context.request::<SetFunctionReturnTypeQuery>(set_function_return_type_query::Variables {
    id,
    export_id: data.export_id,
    ty: ty.to_string()
  }).await?;

  check_errors(&response.errors)?;

  Ok(response.data.unwrap().module.set_function_return_type)
}
