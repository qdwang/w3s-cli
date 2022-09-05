use anyhow::{anyhow, Context, Result};
use clap::Parser;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, ExecutableCommand};
use std::io::{stdout, Stdout};

use std::{
    fs::{self, File},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use w3s::helper;

mod args;
use args::*;

fn print_cli_args(cli_args: &CliArgs, in_alter_screen: bool) -> Result<Stdout> {
    let mut terminal = stdout();
    if in_alter_screen {
        execute!(
            terminal,
            EnterAlternateScreen,
            cursor::MoveTo(0, 0),
            Print(&cli_args),
            cursor::MoveToPreviousLine(1),
            cursor::SavePosition
        )?;
    } else {
        execute!(terminal, Print(format!("{}\n", &cli_args)))?;
    }

    Ok(terminal)
}

fn print_byte_unit(x: usize) -> String {
    if x < 1024 {
        format!("{}B", x)
    } else if x < 1024 * 1024 {
        format!("{:.2}KiB", x as f64 / 1024.)
    } else if x < 1024 * 1024 * 1024 {
        format!("{:.2}MiB", x as f64 / 1024. / 1024.)
    } else if x < 1024 * 1024 * 1024 * 1024 {
        format!("{:.2}GiB", x as f64 / 1024. / 1024. / 1024.)
    } else if x < 1024 * 1024 * 1024 * 1024 * 1024 {
        format!("{:.2}TiB", x as f64 / 1024. / 1024. / 1024. / 1024.)
    } else {
        format!("{:.2}PiB", x as f64 / 1024. / 1024. / 1024. / 1024. / 1024.)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    let result = match cli_args.clone().job {
        Job::Remember(args) => {
            remember_credential(&args.value)?;
            "".to_owned()
        }
        Job::UploadDir(args) => {
            let results = upload_dir(args, cli_args).await?;
            format!("Cid list: {:#?}", results)
        }
        Job::UploadFile(args) => {
            let results = upload_file(args, cli_args).await?;
            format!("Cid list: {:#?}", results)
        }
        Job::DownloadFile(args) => {
            download_file(args, cli_args).await?;
            "".to_owned()
        }
    };

    println!("\n{result}");
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

fn get_progress_listener(mut terminal: Stdout) -> w3s::writer::uploader::ProgressListener {
    Arc::new(Mutex::new(move |_, part, pos, total| {
        let pos = print_byte_unit(pos);
        let total = print_byte_unit(total);

        execute!(
            terminal,
            cursor::RestorePosition,
            cursor::MoveToNextLine(part as u16 + 1),
            Clear(ClearType::CurrentLine),
            Print(format!("part:{part} -> {pos}/{total}"))
        )
        .unwrap();
    }))
}

async fn upload_dir(args: UploadArgs, cli_args: CliArgs) -> Result<Vec<String>> {
    let dir_path = &args.value;
    let max_concurrent = args.max_concurrent;

    let token = get_token()?;

    let terminal = print_cli_args(&cli_args, true)?;

    let results = helper::upload_dir(
        dir_path,
        None,
        token,
        max_concurrent as usize,
        Some(get_progress_listener(terminal)),
        cli_args.with_encryption.map(|x| x.as_bytes().to_vec()),
        if cli_args.with_compression {
            Some(None)
        } else {
            None
        },
    )
    .await?;

    stdout().execute(LeaveAlternateScreen).unwrap();

    Ok(results.iter().map(|x| x.to_string()).collect())
}

async fn upload_file(args: UploadArgs, cli_args: CliArgs) -> Result<Vec<String>> {
    let path = &args.value;
    let max_concurrent = args.max_concurrent;

    let token = get_token()?;

    let terminal = print_cli_args(&cli_args, true)?;

    let results = helper::upload(
        path,
        token,
        max_concurrent as usize,
        Some(get_progress_listener(terminal)),
        Some(None),
        cli_args.with_encryption.map(|x| x.as_bytes().to_vec()),
        if cli_args.with_compression {
            Some(None)
        } else {
            None
        },
    )
    .await?;

    Ok(results.iter().map(|x| x.to_string()).collect())
}

async fn download_file(args: DownloadArgs, cli_args: CliArgs) -> Result<()> {
    let url = &args.value;
    let filename = args.get_target_filename();

    let file = File::create(filename)?;

    let mut terminal = print_cli_args(&cli_args, false)?;

    helper::download(
        url,
        filename,
        file,
        Some(Arc::new(Mutex::new(move |_, _, pos, total| {
            let pos = print_byte_unit(pos);
            let total = print_byte_unit(total);

            execute!(
                terminal,
                cursor::MoveToPreviousLine(1),
                Clear(ClearType::CurrentLine),
                Print(format!("{pos}/{total}\n"))
            )
            .unwrap();
        }))),
        None,
        cli_args.with_encryption.map(|x| x.as_bytes().to_vec()),
        cli_args.with_compression,
    )
    .await?;

    Ok(())
}
