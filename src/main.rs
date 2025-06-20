mod orthanc_types;
mod file_upload_history;

use clap::Parser;
use crossbeam::channel::{bounded, Receiver, Sender};
use orthanc_types::{OrthancErrorResponse, OrthancUploadResponse};
use std::{fs::File, path::PathBuf, thread, time::Duration};
use std::sync::Arc;
use threadpool::ThreadPool;
use walkdir::WalkDir;
use crate::file_upload_history::{FileUploadHistory, NoFileUploadHistory, TextFileUploadHistory};

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

#[derive(Debug)]
struct UploadResult {
    path: PathBuf,
    response: Result<OrthancUploadResponse, OrthancErrorResponse>,
}

fn send_files(
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
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP Client");

            files_rx.iter().for_each(|path| {
                if file_upload_history.already_uploaded(&path) {
                    println!("{} skipped",  path.display());
                } else {
                    let file = File::open(&path).unwrap();
                    let mut request = client.post(format!("{}/instances", url)).body(file);
                    if username.is_some() && password.is_some() {
                        request = request.basic_auth(username.as_ref().unwrap(), password.as_ref());
                    }
                    let response = match request.send() {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Failed request {}", e);
                            return ();
                        }
                    };

                    // println!("{:?}", response.text());
                    // println!("HI");
                    let parsed_response = if response.status().is_success() {
                        let json = response.json::<OrthancUploadResponse>();
                        match json {
                            Ok(x) => Ok(x),
                            Err(_) => Err(OrthancErrorResponse {
                                details: "Failed to parse".to_string(),
                                http_error: "".to_string(),
                                http_status: 0,
                                message: "".to_string(),
                                method: "".to_string(),
                                orthanc_error: "".to_string(),
                                orthanc_status: 0,
                                uri: "".to_string(),
                            }),
                        }
                    } else {
                        let json = response.json();
                        match json {
                            Ok(x) => Err(x),
                            Err(_) => Err(OrthancErrorResponse {
                                details: "Failed to parse".to_string(),
                                http_error: "".to_string(),
                                http_status: 0,
                                message: "".to_string(),
                                method: "".to_string(),
                                orthanc_error: "".to_string(),
                                orthanc_status: 0,
                                uri: "".to_string(),
                            }),
                        }
                    };

                    responses_tx
                        .send(UploadResult {
                            path,
                            response: parsed_response,
                        })
                        .expect("channel will be there waiting for the pool");
                }
            })
        })
    }
}

fn main() {
    let args = Args::parse();

    let file_upload_history: Arc<dyn FileUploadHistory + Send + Sync> = match &args.cache_path {
        Some(path) => Arc::new(TextFileUploadHistory::from_file(path)),
        None => Arc::new(NoFileUploadHistory {})
    };

    let (files_tx, files_rx) = bounded(100);

    thread::spawn(move || {
        WalkDir::new(&args.path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|d| d.path().is_file())
            .for_each(|x| {
                files_tx
                    .send(x.path().to_owned())
                    .expect("channel will be there waiting for the pool");
            })
    });

    let (responses_tx, responses_rx) = bounded(100);
    send_files(
        files_rx,
        responses_tx,
        file_upload_history.clone(),
        &args.url,
        &args.username,
        &args.password,
        args.threads,
    );

    responses_rx.iter().for_each(|upload_result| {
        let status = match &upload_result.response {
            Ok(response) => {
                file_upload_history.on_success(&upload_result.path);
                response.success_message()
            },
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
}
