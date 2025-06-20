mod orthanc_types;
mod file_upload_history;
mod upload;

use crate::file_upload_history::{DisabledFileUploadHistory, FileUploadHistory, TextFileUploadHistory};
use crate::upload::upload_from_channel;
use clap::Parser;
use crossbeam::channel::{bounded, Receiver, Sender};
use std::sync::Arc;
use std::{path::PathBuf, thread};
use threadpool::ThreadPool;
use upload::UploadResult;
use walkdir::WalkDir;

/// Command-line tool to import files into Orthanc.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Username to the REST API
    #[arg(short, long)]
    username: Option<String>,

    /// Password to the REST API
    #[arg(short, long)]
    password: Option<String>,

    /// URL to the REST API of the Orthanc server
    #[arg()]
    url: String,

    /// Be verbose
    #[arg(short, long)]
    verbose: bool,

    /// Number of upload threads
    #[arg(short, long, default_value_t = 4)]
    threads: usize,

    /// Cache file listing file paths already uploaded
    #[arg(short, long)]
    cache_path: Option<PathBuf>,

    /// Directory containing the files to upload
    #[arg()]
    path: PathBuf,
}

fn start_send_files(
    files_rx: Receiver<PathBuf>,
    responses_tx: Sender<UploadResult>,
    file_upload_history: Arc<dyn FileUploadHistory + Send + Sync>,
    url: &str,
    username: &Option<String>,
    password: &Option<String>,
    threads: usize,
) {
    let pool = ThreadPool::new(threads);

    for _ in 0..threads {
        let files_rx = files_rx.clone();
        let responses_tx = responses_tx.clone();
        let file_upload_history =  file_upload_history.clone();
        let url = url.to_owned();
        let username = username.clone().to_owned();
        let password = password.clone().to_owned();
        pool.execute(move || {
            upload_from_channel(
                files_rx,
                responses_tx,
                file_upload_history,
                url,
                username,
                password
            )
        })
    }
}

fn walk_dir(path: PathBuf, files_tx: Sender<PathBuf>) {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|d| d.path().is_file())
        .for_each(|x| {
            files_tx
                .send(x.path().to_owned())
                .expect("channel will be there waiting for the pool");
        })
}

fn main() {
    let args = Args::parse();

    let file_upload_history: Arc<dyn FileUploadHistory + Send + Sync> = match &args.cache_path {
        Some(path) => Arc::new(TextFileUploadHistory::from_file(path)),
        None => Arc::new(DisabledFileUploadHistory {})
    };

    let (files_tx, files_rx) = bounded(100);
    let (responses_tx, responses_rx) = bounded(100);

    thread::spawn(move || walk_dir(args.path, files_tx));
    start_send_files(
        files_rx,
        responses_tx,
        file_upload_history.clone(),
        &args.url,
        &args.username,
        &args.password,
        args.threads,
    );
    
    let mut successes = 0;
    let mut failures = 0;
    responses_rx.iter().for_each(|upload_result| {
        if upload_result.response.is_ok() {
            file_upload_history.on_success(&upload_result.path);
            successes += 1;
        } else {
            failures += 1;
        }
        let status = match &upload_result.response {
            Ok(response) => response.success_message(),
            Err(_) => "Error".to_string(),
        };
        println!("{}: {}", upload_result.path.to_string_lossy(), status);
        if args.verbose || upload_result.response.is_err() {
            match upload_result.response {
                Ok(response) => println!("{}", response),
                Err(err) => println!("{}", err),
            }
        }
    });
    
    println!("Successes: {}", successes);
    println!("Failures: {}", failures);
}
