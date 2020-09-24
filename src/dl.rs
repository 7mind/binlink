use std::{
    fs,
    io::{self, copy, Read},
    path::Path,

};

use exitfailure::ExitFailure;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{header, Url};
use reqwest::blocking as req;

struct DownloadProgress<R> {
    inner: R,
    progress_bar: ProgressBar,
}

impl<R: Read> Read for DownloadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}

fn download(url: &str) -> Result<(), ExitFailure> {
    let url = Url::parse(url)?;
    let client = req::Client::new();

    let request = client.get(url.as_str());

    let file = Path::new(
        url
            .path_segments()
            .and_then(|segments| segments.last())
            .unwrap_or("tmp.bin"),
    );

    let pb = ProgressBar::new(1000);

    let mut dest = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)?;


    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .progress_chars("#>-"));

    let resp = request.send()?;

    let mut source = DownloadProgress {
        progress_bar: pb,
        inner: resp,
    };


    let sz = source.inner.headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|ct_len| ct_len.to_str().ok())
        .and_then(|ct_len| ct_len.parse().ok())
        .unwrap_or(0);

    source.progress_bar.set_length(sz);

    let _ = copy(&mut source, &mut dest)?;

    println!(
        "Download of '{}' has been completed.",
        file.to_str().unwrap()
    );

    Ok(())
}
