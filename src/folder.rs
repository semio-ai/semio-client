
use clap::{Parser, Subcommand};
use uuid::Uuid;
use semio_record::{folder::v0::{Folder, unfrozen, public}, record::RecordDefn, acl::Acl};
use crate::{context::Context, common::*};

macro_rules! impl_public_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Folder as RecordDefn>::Public {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          parent: value.parent,
          name: value.name,
        })
      }
    }
  }
}

macro_rules! impl_private_from_response {
  ($query: path) => {
    impl FromResponse<$query> for <Folder as RecordDefn>::Private {
      fn from_response(value: $query) -> anyhow::Result<Self> {
        Ok(Self {
          parent: value.parent,
          name: value.name,
          acl: Acl::from_response(value.acl)?,
        })
      }
    }
  }
}

impl_public_from_response!(get_public_query::GetPublicQueryFolderGetPublic);

impl_private_from_response!(get_private_query::GetPrivateQueryFolderGetPrivate);
impl_permission_level_from_response!(get_private_query::PermissionLevel);
impl_with_permissions_from_response!(get_private_query::GetPrivateQueryFolderGetPrivateAclPermissionsWithPermissions);
impl_with_permissions_from_response!(get_private_query::GetPrivateQueryFolderGetPrivateAclDefault);
impl_acl_from_response!(get_private_query::GetPrivateQueryFolderGetPrivateAcl);

impl_public_from_response!(get_version_public_query::GetVersionPublicQueryFolderGetVersionPublic);

impl_private_from_response!(get_version_private_query::GetVersionPrivateQueryFolderGetVersionPrivate);
impl_permission_level_from_response!(get_version_private_query::PermissionLevel);
impl_with_permissions_from_response!(get_version_private_query::GetVersionPrivateQueryFolderGetVersionPrivateAclPermissionsWithPermissions);
impl_with_permissions_from_response!(get_version_private_query::GetVersionPrivateQueryFolderGetVersionPrivateAclDefault);
impl_acl_from_response!(get_version_private_query::GetVersionPrivateQueryFolderGetVersionPrivateAcl);

impl_create!(folder, "src/folder/create.graphql");
impl_get_public!(<Folder as RecordDefn>::Public, folder, "src/folder/get_public.graphql");
impl_get_private!(<Folder as RecordDefn>::Private, folder, "src/folder/get_private.graphql");
impl_get_version_public!(<Folder as RecordDefn>::Public, folder, "src/folder/get_version_public.graphql");
impl_get_version_private!(<Folder as RecordDefn>::Private, folder, "src/folder/get_version_private.graphql");
impl_set_name!(folder, "src/folder/set_name.graphql");
impl_set_parent!(folder, "src/folder/set_parent.graphql");
impl_add_permissions!(folder, "src/folder/add_permissions.graphql");
impl_set_permissions!(folder, "src/folder/set_permissions.graphql");
impl_remove_permissions!(folder, "src/folder/remove_permissions.graphql");
impl_set_default_permissions!(folder, "src/folder/set_default_permissions.graphql");
