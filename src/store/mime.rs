use std::collections::HashMap;

use lazy_static::lazy_static;


macro_rules! mime_types {
  ($($extension:literal => $mime:literal),*) => {{
    let mut mime_types = std::collections::HashMap::new();
    $(
      mime_types.insert($extension, $mime);
    )*
    mime_types
  }};
}

lazy_static! {
  pub static ref MIME_TYPES: HashMap<&'static str, &'static str> = mime_types!(
    "txt" => "text/plain",
    "html" => "text/html",
    "css" => "text/css",
    "js" => "application/javascript",
    "json" => "application/json",
    "png" => "image/png",
    "jpg" => "image/jpeg",
    "jpeg" => "image/jpeg",
    "gif" => "image/gif",
    "svg" => "image/svg+xml",
    "ico" => "image/x-icon",
    "woff" => "application/font-woff",
    "woff2" => "application/font-woff2",
    "ttf" => "application/font-ttf",
    "otf" => "application/font-otf",
    "eot" => "application/vnd.ms-fontobject",
    "mp3" => "audio/mpeg",
    "mp4" => "video/mp4",
    "webm" => "video/webm",
    "ogg" => "video/ogg",
    "ogv" => "video/ogg",
    "oga" => "audio/ogg",
    "ogx" => "application/ogg",
    "pdf" => "application/pdf",
    "doc" => "application/msword",
    "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "xls" => "application/vnd.ms-excel",
    "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "ppt" => "application/vnd.ms-powerpoint",
    "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    "zip" => "application/zip",
    "tar" => "application/x-tar",
    "gz" => "application/gzip",
    "rar" => "application/x-rar-compressed",
    "7z" => "application/x-7z-compressed",
    "bz2" => "application/x-bzip2",
    "bz" => "application/x-bzip",
    "xz" => "application/x-xz",
    "tgz" => "application/x-gzip",
    "gz" => "application/x-gzip",
    "tar" => "application/x-tar"
  );

  pub static ref MIME_TYPE_EXTENSIONS: HashMap<&'static str, &'static str> = {
    let mut ret = HashMap::new();
    for (extension, mime_type) in MIME_TYPES.iter() {
      ret.insert(*mime_type, *extension);
    }
    ret
  };
    
}