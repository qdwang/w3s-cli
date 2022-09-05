use std::fmt::Display;

use clap::{Args, Parser, Subcommand};

#[derive(Subcommand, Clone)]
#[clap(disable_help_subcommand = true)]
pub enum Job {
    /// Remember the web3.storage API token
    Remember(GeneralArgs),
    /// Upload a directory
    UploadDir(UploadArgs),
    /// Upload a file
    UploadFile(UploadArgs),
    /// Download a file from IPFS gateway
    DownloadFile(DownloadArgs),
}

#[derive(Args, Clone)]
pub struct GeneralArgs {
    #[clap(value_parser)]
    pub value: String,
}
#[derive(Args, Clone)]
pub struct UploadArgs {
    #[clap(value_parser)]
    pub value: String,
    #[clap(short, long, value_parser, default_value_t = 1)]
    pub max_concurrent: u8,
}
#[derive(Args, Clone)]
pub struct DownloadArgs {
    #[clap(value_parser)]
    pub value: String,
    #[clap(value_parser)]
    pub to_file_name: Option<String>,
}
impl DownloadArgs {
    pub fn get_target_filename(&self) -> &str {
        if let Some(x) = self.to_file_name.as_ref() {
            x.as_str()
        } else {
            self.value.split('/').last().unwrap_or("downloaded")
        }
    }
}

impl Display for GeneralArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
impl Display for UploadArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n  max concurrent -> {}", self.value, self.max_concurrent)
    }
}
impl Display for DownloadArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n  save to file -> {}", self.value, self.get_target_filename())
    }
}

#[derive(Parser, Clone)]
#[clap(version, author, about)]
#[clap(disable_version_flag = true, propagate_version = true)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub job: Job,

    /// Work with encryption/decryption
    #[clap(short = 'e', long, id = "PASSWORD")]
    pub with_encryption: Option<String>,

    /// Work with compression/decompression
    #[clap(short = 'c', long, action)]
    pub with_compression: bool,
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
