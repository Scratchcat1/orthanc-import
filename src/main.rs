use clap::Parser;
use crossbeam::channel::bounded;
use orthanc_import::file_history::disabled::DisabledFileUploadHistory;
use orthanc_import::file_history::sqlite::SqliteFileUploadHistory;
use orthanc_import::file_history::util::FileUploadHistory;
use orthanc_import::file_history::FileUploadHistoryImpl;
use orthanc_import::orthanc::connector::OrthancConnector;
use orthanc_import::threading::upload::upload_from_channel;
use orthanc_import::threading::walk_tree::walk_dir;
use r2d2_sqlite::SqliteConnectionManager;
use std::{path::PathBuf, thread};
use threadpool::ThreadPool;

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

fn main() {
    let args = Args::parse();

    let connector = OrthancConnector::new(args.username, args.password, args.url);

    let file_history: FileUploadHistoryImpl = match &args.cache_path {
        Some(path) => {
            let manager = SqliteConnectionManager::file(path);
            let pool = r2d2::Pool::new(manager).unwrap();
            FileUploadHistoryImpl::Sqlite(SqliteFileUploadHistory::new(pool))
        }
        None => FileUploadHistoryImpl::Disabled(DisabledFileUploadHistory::default()),
    };

    let (files_tx, files_rx) = bounded(100);
    let (responses_tx, responses_rx) = bounded(100);

    thread::spawn(move || walk_dir(args.path, files_tx));

    let pool = ThreadPool::new(args.threads);
    for _ in 0..pool.max_count() {
        let connector = connector.clone();
        let file_history = file_history.clone();
        let files_rx = files_rx.clone();
        let responses_tx = responses_tx.clone();
        pool.execute(move || upload_from_channel(&connector, file_history, files_rx, responses_tx))
    }

    let mut successes = 0;
    let mut failures = 0;
    responses_rx.iter().for_each(|upload_result| {
        file_history.on_upload(&upload_result.id);
        if upload_result.response.is_ok() {
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
