use std::fmt::Display;

use clap::{Args, Parser, Subcommand};

#[derive(Subcommand, Clone)]
#[clap(disable_help_subcommand = true)]
pub enum Job {
    /// Remember the web3.storage API token
    Remember(GeneralArgs),
    /// Upload a directory
    UploadDir(GeneralArgs),
    /// Upload a file
    UploadFile(GeneralArgs),
    /// Download a file from cid
    DownloadFile(GeneralArgs),
}

#[derive(Args, Clone)]
pub struct GeneralArgs {
    #[clap(value_parser)]
    pub value: String,
}

#[derive(Parser, Clone)]
#[clap(version, author, about)]
#[clap(disable_version_flag = true, propagate_version = true)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub job: Job,

    /// Upload/download with encryption/decryption
    #[clap(short = 'e', long)]
    pub with_encryption: Option<String>,

    /// Upload/download with compression/decompression (useful for text contents)
    #[clap(short = 'c', long, action)]
    pub with_compression: bool,
}

impl Display for GeneralArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Job::Remember(args) => format!("Remember API token: {}", args),
                Job::DownloadFile(args) => format!("Download file from: {}", args),
                Job::UploadDir(args) => format!("Upload this directory: {}", args),
                Job::UploadFile(args) => format!("Upload this file: {}", args),
            }
        )
    }
}

impl Display for CliArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (enc_label, comp_lebel) = match self.job {
            Job::DownloadFile(_) => ("decryption", "decompression"),
            _ => ("encryption", "compression"),
        };

        write!(
            f,
            "Arguments used:\n  Job -> {}\n  {} -> {}\n  {} -> {}\n\n",
            self.job,
            enc_label,
            match self.with_encryption.as_ref() {
                Some(pwd) => format!("password length: {}", pwd.len()),
                None => "false".to_owned(),
            },
            comp_lebel,
            self.with_compression
        )
    }
}