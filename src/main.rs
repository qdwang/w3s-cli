use anyhow::{anyhow, Context, Result};
use byte_unit::Byte;
use clap::Parser;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, ExecutableCommand};
use std::io::stdout;

use std::{
    fs::{self, File},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use w3s::helper;

mod args;
use args::{CliArgs, Job};

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    execute!(
        stdout(),
        EnterAlternateScreen,
        cursor::MoveTo(0, 0),
        Print(&cli_args)
    )?;

    let result = match cli_args.clone().job {
        Job::Remember(args) => {
            remember_credential(&args.value)?;
            "".to_owned()
        }
        Job::UploadDir(args) => {
            let results = upload_dir(&args.value, cli_args).await?;
            format!("Cid list: {:#?}", results)
        }
        Job::UploadFile(args) => {
            let results = upload_file(&args.value, cli_args).await?;
            format!("Cid list: {:#?}", results)
        }
        Job::DownloadFile(args) => {
            download_file(&args.value, cli_args).await?;
            "".to_owned()
        }
    };

    stdout().execute(LeaveAlternateScreen).unwrap();

    println!("\n{result}\n=== DONE ===");
    Ok(())
}

fn credentials_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("unable to get the home dir path")?;
    let w3s_dir = home_dir.join(".w3s");
    if !w3s_dir.is_dir() {
        fs::create_dir(&w3s_dir)?;
    }
    let credentials = w3s_dir.join("credentials");
    if !credentials.is_file() {
        fs::write(&credentials, "")?;
    }
    Ok(credentials)
}
fn remember_credential(token: &str) -> Result<()> {
    let credentials_path = credentials_path()?;
    fs::write(credentials_path, token)?;
    Ok(())
}
fn get_token() -> Result<String> {
    let credentials_path = credentials_path()?;
    let token = fs::read_to_string(credentials_path)?;
    if token.is_empty() {
        Err(anyhow!(
            "No API token found, please remember API token first"
        ))
    } else {
        Ok(token)
    }
}

async fn upload_dir(path: &str, mut cli_args: CliArgs) -> Result<Vec<String>> {
    let token = get_token()?;

    let mut terminal = stdout();

    let results = helper::upload_dir(
        path,
        None,
        token,
        8,
        Some(Arc::new(Mutex::new(move |_, part, pos, total| {
            let pos = Byte::from_bytes(pos as u128).get_appropriate_unit(true);
            let total = Byte::from_bytes(total as u128).get_appropriate_unit(true);

            execute!(
                terminal,
                cursor::MoveTo(0, part as u16 + 5),
                Clear(ClearType::CurrentLine),
                Print(format!("part:{part} -> {pos}/{total}"))
            )
            .unwrap();
        }))),
        cli_args
            .with_encryption
            .as_mut()
            .map(|x| unsafe { x.as_bytes_mut() }),
        if cli_args.with_compression {
            Some(None)
        } else {
            None
        },
    )
    .await?;

    Ok(results.iter().map(|x| x.to_string()).collect())
}

async fn upload_file(path: &str, mut cli_args: CliArgs) -> Result<Vec<String>> {
    let token = get_token()?;

    let results = helper::upload(
        path,
        token,
        4,
        None,
        Some(None),
        cli_args
            .with_encryption
            .as_mut()
            .map(|x| unsafe { x.as_bytes_mut() }),
        if cli_args.with_compression {
            Some(None)
        } else {
            None
        },
    )
    .await?;

    Ok(results.iter().map(|x| x.to_string()).collect())
}

async fn download_file(path: &str, cli_args: CliArgs) -> Result<()> {
    let name = path.split('/').last().unwrap_or("downloaded");
    let file = File::open(name)?;

    helper::download(
        path,
        name,
        file,
        None,
        None,
        cli_args
            .with_encryption
            .map(|x| x.as_bytes().iter().copied().collect::<Vec<_>>()),
        cli_args.with_compression,
    )
    .await?;

    Ok(())
}
