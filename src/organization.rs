use clap::{Parser, Subcommand};
use semio_record::{organization::v0::{Organization, unfrozen}, record::RecordDefn};
use semio_record::acl::Acl;
use uuid::Uuid;
use crate::{context::Context, common::*};

macro_rules! impl_public_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Organization as RecordDefn>::Public {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          name: value.name,
        })
      }
    }
  }
}

macro_rules! impl_private_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Organization as RecordDefn>::Private {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          name: value.name,
          acl: Acl::from_response(value.acl)?,
        })
      }
    }
  }
}



impl_public_from_response!(get_public_query::GetPublicQueryOrganizationGetPublic);

impl_private_from_response!(get_private_query::GetPrivateQueryOrganizationGetPrivate);
impl_permission_level_from_response!(get_private_query::PermissionLevel);
impl_with_permissions_from_response!(get_private_query::GetPrivateQueryOrganizationGetPrivateAclPermissionsWithPermissions);
impl_with_permissions_from_response!(get_private_query::GetPrivateQueryOrganizationGetPrivateAclDefault);
impl_acl_from_response!(get_private_query::GetPrivateQueryOrganizationGetPrivateAcl);

impl_public_from_response!(get_version_public_query::GetVersionPublicQueryOrganizationGetVersionPublic);

impl_private_from_response!(get_version_private_query::GetVersionPrivateQueryOrganizationGetVersionPrivate);
impl_permission_level_from_response!(get_version_private_query::PermissionLevel);
impl_with_permissions_from_response!(get_version_private_query::GetVersionPrivateQueryOrganizationGetVersionPrivateAclPermissionsWithPermissions);
impl_with_permissions_from_response!(get_version_private_query::GetVersionPrivateQueryOrganizationGetVersionPrivateAclDefault);
impl_acl_from_response!(get_version_private_query::GetVersionPrivateQueryOrganizationGetVersionPrivateAcl);

impl_create!(organization, "src/organization/create.graphql");
impl_get_public!(<Organization as RecordDefn>::Public, organization, "src/organization/get_public.graphql");
impl_get_private!(<Organization as RecordDefn>::Private, organization, "src/organization/get_private.graphql");
impl_get_version_public!(<Organization as RecordDefn>::Public, organization, "src/organization/get_version_public.graphql");
impl_get_version_private!(<Organization as RecordDefn>::Private, organization, "src/organization/get_version_private.graphql");
impl_set_name!(organization, "src/organization/set_name.graphql");
impl_add_permissions!(organization, "src/organization/add_permissions.graphql");
impl_set_permissions!(organization, "src/organization/set_permissions.graphql");
impl_remove_permissions!(organization, "src/organization/remove_permissions.graphql");
impl_set_default_permissions!(organization, "src/organization/set_default_permissions.graphql");
