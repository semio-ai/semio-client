use std::{collections::HashMap, io::SeekFrom, path::PathBuf};

use chrono::Utc;
use clap::Parser;
use futures::StreamExt;
use semio_record::acl::{Acl, PermissionLevel, Permissions, WithPermissions};
use semio_store_rpc::{client::connect, Chunk, Metadata};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{fs::File, io::AsyncSeekExt};
use uuid::Uuid;

use crate::context::Context;

pub mod mime;

#[derive(Debug, Parser)]
pub struct Upload {
  pub file: String,

  /// The MIME type of the file. If unspecified, the type is guessed from the file extension.
  #[clap(short, long)]
  pub mime: Option<String>,

  /// The name of the file. If unspecified, the name is guessed from the file name.
  #[clap(short, long)]
  pub name: Option<String>,

  /// Chunk size (in bytes)
  #[clap(short, long, default_value = "1240000")]
  pub chunk_size: u64,
}

#[derive(Debug, Parser)]
pub struct Download {
  /// The UUID of the file.
  #[clap(long)]
  pub id: String,

  /// The output file path. If unspecified, the file's name and MIME type extension is used.
  #[clap(short, long)]
  pub output: Option<String>,
}

fn store_url(context: &Context) -> String {
  format!("ws://{}/store", context.url)
}

pub async fn upload<'a>(context: &Context, upload: Upload) -> anyhow::Result<()> {
  let client = connect(store_url(context)).await?;
  let Upload {
    file,
    mime,
    name,
    chunk_size,
  } = upload;
  let file = PathBuf::from(file);

  let file_name = if let Some(file_name) = file.file_name() {
    file_name.to_str()
  } else {
    anyhow::bail!("File name is not specified and cannot be guessed");
  };

  let file_name = file_name.ok_or_else(|| anyhow::anyhow!("File name is not valid UTF-8"))?;

  let name = if let Some(name) = name {
    name
  } else {
    // Safety: The split should have at least one component
    file_name.split('.').next().unwrap().to_string()
  };

  let mime_type = if let Some(mime_type) = mime {
    mime_type
  } else {
    let extension = file_name
      .split('.')
      .skip(1)
      .collect::<Vec<&str>>()
      .join(".");
    if let Some(mime_type) = mime::MIME_TYPES.get(extension.as_str()) {
      mime_type.to_string()
    } else {
      anyhow::bail!("MIME type is not specified and cannot be guessed");
    }
  };

  let size = file.metadata()?.len();

  let metadata = Metadata {
    name,
    mime_type,
    chunk_size,
    created_at: Utc::now(),
    creator_id: Uuid::nil(),
    size,
    acl: Acl {
      default: WithPermissions::Custom(Permissions {
        read: PermissionLevel::Private,
        write: PermissionLevel::Private,
      }),
      permissions: HashMap::new(),
    },
  };

  let id = client
    .create(metadata)
    .await
    .map_err(|e| anyhow::anyhow!("{}", e))?;
  println!("{}", id);

  let mut file = File::open(&file).await?;
  let mut buffer = vec![0u8; chunk_size as usize];

  let full_chunks = size / chunk_size;
  let last_chunk_size = size % chunk_size;

  for offset in 0..full_chunks {
    file.read_exact(&mut buffer).await?;
    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let hash = hasher.finalize();
    let chunk = Chunk {
      offset,
      data: buffer.into_boxed_slice(),

      // Safety: The hash is always 32 bytes long
      checksum: hash.try_into().unwrap(),
    };
    client
      .upload_chunk(id, chunk)
      .await
      .map_err(|e| anyhow::anyhow!("{}", e))?;
    buffer = vec![0u8; chunk_size as usize];
  }

  if last_chunk_size > 0 {
    file
      .read_exact(&mut buffer[..last_chunk_size as usize])
      .await?;
    let mut hasher = Sha256::new();
    hasher.update(&buffer[..last_chunk_size as usize]);
    let hash = hasher.finalize();
    let chunk = Chunk {
      offset: full_chunks,
      data: buffer.into_boxed_slice(),

      // Safety: The hash is always 32 bytes long
      checksum: hash.try_into().unwrap(),
    };
    client
      .upload_chunk(id, chunk)
      .await
      .map_err(|e| anyhow::anyhow!("{}", e))?;
  }

  Ok(Default::default())
}

pub async fn download(context: &Context, download: Download) -> anyhow::Result<()> {
  let client = connect(store_url(context)).await?;
  let Download { id, output } = download;
  let id = Uuid::parse_str(id.as_str())?;
  let mut download = client
    .download(id)
    .await
    .map_err(|e| anyhow::anyhow!("{}", e))?;

  let Metadata {
    name,
    mime_type,
    chunk_size,
    size,
    ..
  } = download.metadata;

  let output = if let Some(output) = output {
    output
  } else {
    let extension = mime::MIME_TYPE_EXTENSIONS.get(mime_type.as_str());
    if let Some(extension) = extension {
      format!("{}.{}", name, extension)
    } else {
      anyhow::bail!("MIME type extension is not specified and cannot be guessed");
    }
  };

  let mut file = File::create(&output).await?;
  file.set_len(size).await?;
  while let Some(chunk_res) = download.stream.next().await {
    let chunk = chunk_res.map_err(|e| anyhow::anyhow!("{}", e))?;

    // TODO: Verify checksum

    file
      .seek(SeekFrom::Start(chunk.offset * chunk_size))
      .await?;
    file.write_all(&chunk.data).await?;
  }

  Ok(())
}
