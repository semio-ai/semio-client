use std::os::unix::prelude::AsRawFd;

use tokio::io::{stdin, stdout, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct ReadLineOptions<'a, W: AsyncWrite> {
  pub write: &'a mut W,
  pub echo: bool,
}

pub async fn read_line<'a, R: AsyncRead + Unpin, W: AsyncWrite + AsRawFd + Unpin>(
  r: &mut R,
  options: ReadLineOptions<'a, W>,
) -> tokio::io::Result<String> {
  let mut buf = [0u8; 1];
  let mut ret = String::new();

  use termios::*;

  if !options.echo {
    let fd = options.write.as_raw_fd();
    let mut termios = Termios::from_fd(fd)?;
    termios.c_lflag &= !ECHO;

    tcsetattr(fd, TCSAFLUSH, &termios)?;
  }

  while let Ok(len) = r.read_exact(&mut buf).await {
    if len == 0 {
      break;
    }

    match buf[0] {
      b'\n' => break,
      b'\r' => continue,
      _ => (),
    }

    ret.push(buf[0] as char);
  }

  if !options.echo {
    let fd = options.write.as_raw_fd();
    let mut termios = Termios::from_fd(fd)?;
    termios.c_lflag |= ECHO;

    tcsetattr(fd, TCSAFLUSH, &termios)?;

    if cfg!(windows) {
      options.write.write_all(b"\r\n").await?;
    } else {
      options.write.write_all(b"\n").await?;
    }
    options.write.flush().await?;
  }

  Ok(ret)
}

pub struct PromptOptions<'a, W: AsyncWrite> {
  pub prompt: String,
  pub write: &'a mut W,
  pub echo: bool,
}

pub async fn prompt<'a, R: AsyncRead + Unpin, W: AsyncWrite + Unpin + AsRawFd>(
  r: &mut R,
  options: PromptOptions<'a, W>,
) -> tokio::io::Result<String> {
  options.write.write_all(options.prompt.as_bytes()).await?;
  options.write.write_all(b": ").await?;
  options.write.flush().await?;

  Ok(
    read_line(
      r,
      ReadLineOptions {
        write: options.write,
        echo: options.echo,
      },
    )
    .await?,
  )
}

pub struct StdPromptOptions {
  pub prompt: String,
  pub echo: bool,
}

pub async fn std_prompt(options: StdPromptOptions) -> tokio::io::Result<String> {
  Ok(
    prompt(
      &mut stdin(),
      PromptOptions {
        prompt: options.prompt,
        write: &mut stdout(),
        echo: options.echo,
      },
    )
    .await?,
  )
}
