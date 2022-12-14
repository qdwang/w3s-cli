use anyhow::{anyhow, Context, Result};
use clap::Parser;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, execute};
use std::collections::{HashMap, VecDeque};
use std::io::stdout;

use std::{
    fs::{self, File},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use w3s::helper;
use w3s::writer::car_util;

mod args;
use args::*;

fn print_byte_unit(x: usize) -> String {
    let mut suffix = VecDeque::from(["B", "KiB", "MiB", "GiB", "TiB", "PiB"]);

    let mut x = x as f32;
    while x >= 1024. {
        suffix.pop_front();
        x /= 1024.;
    }

    format!("{:.3}{}", x, suffix.pop_front().unwrap_or(">PiB"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    let with_encryption = if cli_args.with_encryption {
        fn input_password() -> Result<String> {
            println!("Input password: ");
            let pwd = rpassword::read_password()?;
            println!("Input password again: ");
            let pwd2 = rpassword::read_password()?;
            if pwd == pwd2 {
                Ok(pwd)
            } else {
                println!("Two passwords do not match.\n");
                input_password()
            }
        }

        Some(input_password()?.as_bytes().to_vec())
    } else {
        None
    };

    println!("{}", &cli_args);

    let (results, has_empty_cid) = match cli_args.clone().job {
        Job::Remember(args) => {
            remember_credential(&args.value)?;
            (vec![], None)
        }
        Job::UploadDir(args) => upload_dir(args, cli_args, with_encryption).await?,
        Job::UploadFile(args) => upload_file(args, cli_args, with_encryption).await?,
        Job::DownloadFile(args) => {
            download_file(args, cli_args, with_encryption).await?;
            (vec![], None)
        }
        Job::DownloadDir(args) => {
            download_dir(args, cli_args, with_encryption).await?;
            (vec![], None)
        }
    };

    if !results.is_empty() {
        println!("\n{:#?}", results);

        if let Some(cid) = has_empty_cid {
            println!("\nThis cid({cid}) is temporarily used for streaming uploads and you can delete it from the web3.storage project list page.");
        }
    }

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

fn get_progress_listener() -> w3s::writer::uploader::ProgressListener {
    let mut terminal = stdout();
    let mut parts: HashMap<usize, (usize, usize, bool)> = HashMap::new();

    Arc::new(Mutex::new(move |_, part, pos, total| {
        parts.insert(part, (pos, total, false));

        let mut part_display_lst = Vec::with_capacity(32);
        let mut sum_pos = 0;
        let mut sum_total = 0;

        for (part, (pos, total, is_finished)) in parts.iter() {
            if !is_finished {
                part_display_lst.push(part);
            }
            sum_pos += pos;
            sum_total += total;
        }

        part_display_lst.sort();

        let content = format!(
            "[{}/{}] {}\n",
            print_byte_unit(sum_pos),
            print_byte_unit(sum_total),
            part_display_lst
                .iter()
                .map(|x| format!("{x} "))
                .collect::<String>()
        );

        execute!(
            terminal,
            cursor::MoveToPreviousLine(1),
            Clear(ClearType::CurrentLine),
            Print(content)
        )
        .unwrap();

        if pos == total {
            parts.insert(part, (pos, total, true));
        }
    }))
}

async fn upload_dir(
    args: UploadArgs,
    cli_args: CliArgs,
    with_encryption: Option<Vec<u8>>,
) -> Result<(Vec<String>, Option<String>)> {
    let dir_path = &args.value;
    let max_concurrent = args.max_concurrent;

    let token = get_token()?;

    let results = helper::upload_dir(
        dir_path,
        None,
        token,
        max_concurrent as usize,
        Some(get_progress_listener()),
        with_encryption,
        if cli_args.with_compression {
            Some(None)
        } else {
            None
        },
    )
    .await?;

    let cid_string_lst = results.iter().map(|x| x.to_string()).collect();
    Ok((
        cid_string_lst,
        car_util::find_empty_item(&results).map(|x| x.to_string()),
    ))
}

async fn upload_file(
    args: UploadArgs,
    cli_args: CliArgs,
    with_encryption: Option<Vec<u8>>,
) -> Result<(Vec<String>, Option<String>)> {
    let path = &args.value;
    let max_concurrent = args.max_concurrent;

    let token = get_token()?;

    let results = helper::upload(
        path,
        token,
        max_concurrent as usize,
        Some(get_progress_listener()),
        Some(None),
        with_encryption,
        if cli_args.with_compression {
            Some(None)
        } else {
            None
        },
    )
    .await?;

    let cid_string_lst = results.iter().map(|x| x.to_string()).collect();
    Ok((
        cid_string_lst,
        car_util::find_empty_item(&results).map(|x| x.to_string()),
    ))
}

async fn download_dir(
    args: DownloadArgs,
    cli_args: CliArgs,
    with_encryption: Option<Vec<u8>>,
) -> Result<()> {
    let url = &args.value;
    let dir_path = args.to_path.unwrap_or_else(|| "w3s_downloaded".to_owned());

    let mut terminal = stdout();
    let mut downloading_name = Arc::new("".to_owned());

    helper::download_dir(
        url,
        &dir_path,
        Some(|url, status| println!("checked: {url} -> {status}\n")),
        Some(Arc::new(Mutex::new(move |name, _, pos, total| {
            let pos = print_byte_unit(pos);
            let total = print_byte_unit(total);

            if downloading_name != name {
                println!();
                downloading_name = name.clone();
            }

            execute!(
                terminal,
                cursor::MoveToPreviousLine(1),
                Clear(ClearType::CurrentLine),
                Print(format!("[{pos}/{total}] {name}\n"))
            )
            .unwrap();
        }))),
        with_encryption,
        cli_args.with_compression,
    )
    .await?;

    Ok(())
}

async fn download_file(
    args: DownloadArgs,
    cli_args: CliArgs,
    with_encryption: Option<Vec<u8>>,
) -> Result<()> {
    let url = &args.value;
    let filename = args.get_target_filename();

    let file = File::create(filename)?;

    let mut terminal = stdout();

    helper::download(
        url,
        filename,
        file,
        Some(Arc::new(Mutex::new(move |name, _, pos, total| {
            let pos = print_byte_unit(pos);
            let total = print_byte_unit(total);

            execute!(
                terminal,
                cursor::MoveToPreviousLine(1),
                Clear(ClearType::CurrentLine),
                Print(format!("[{pos}/{total}] {name}\n"))
            )
            .unwrap();
        }))),
        None,
        with_encryption,
        cli_args.with_compression,
    )
    .await?;

    Ok(())
}
