mod orthanc_types;

use clap::Parser;
use crossbeam::channel::{bounded, Receiver, Sender};
use orthanc_types::{OrthancErrorResponse, OrthancUploadResponse};
use std::{fs::File, path::PathBuf, thread, time::Duration};
use threadpool::ThreadPool;
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
    url: &str,
    username: &Option<String>,
    password: &Option<String>,
    threads: usize,
) {
    let pool = ThreadPool::new(threads);

    for _ in 0..threads {
        let files_rx = files_rx.clone();
        let responses_tx = responses_tx.clone();
        let url = url.clone().to_owned();
        let username = username.clone().to_owned();
        let password = password.clone().to_owned();
        pool.execute(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP Client");

            files_rx.iter().for_each(|path| {
                let file = File::open(path.clone()).unwrap();
                let mut request = client.post(format!("{}/instances", url)).body(file);
                if username.is_some() && password.is_some() {
                    request = request.basic_auth(username.as_ref().unwrap(), password.as_ref());
                }
                let response = request
                    .send()
                    .expect("An error occured while making request");

                // println!("{:?}", response.text());
                // println!("HI");
                let parsed_response = if response.status().is_success() {
                    Ok(response
                        .json::<OrthancUploadResponse>()
                        .expect("Could not parse server response"))
                } else {
                    Err(response
                        .json()
                        .expect("Expected response to have parseable body"))
                };

                responses_tx
                    .send(UploadResult {
                        path,
                        response: parsed_response,
                    })
                    .expect("channel will be there waiting for the pool");
            })
        })
    }
}

fn main() {
    let args = Args::parse();

    let (files_tx, files_rx) = bounded(100);

    thread::spawn(move || {
        WalkDir::new(&args.path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|d| d.path().is_file())
            .for_each(|x| {
                let files_tx = files_tx.clone();
                let filepath = x.path().to_owned();
                files_tx
                    .send(filepath)
                    .expect("channel will be there waiting for the pool");
            })
    });

    let (responses_tx, responses_rx) = bounded(100);
    send_files(
        files_rx,
        responses_tx,
        &args.url,
        &args.username,
        &args.password,
        args.threads,
    );

    responses_rx.iter().for_each(|upload_result| {
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
    // std::thread::sleep(Duration::from_secs(60));
}
