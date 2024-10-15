use std::fs;
use std::io::{self, BufRead, Write};
use tokio::task;
use tokio::sync::Semaphore;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};


mod func;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (webpath, endpoint_input, threads) = func::parse_args();

    let webpath = Arc::new(webpath);
    let semaphore = Arc::new(Semaphore::new(threads));
    let progress = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    if let Ok(file) = fs::File::open(&endpoint_input) {
        let reader = io::BufReader::new(file);
        let endpoints: Vec<_> = reader.lines().collect::<Result<_, _>>()?;
        let total = endpoints.len();

        for endpoint in endpoints {
            let webpath = Arc::clone(&webpath);
            let semaphore = Arc::clone(&semaphore);
            let progress = Arc::clone(&progress);

            let handle = task::spawn(async move {
                let permit = semaphore.acquire_owned().await.unwrap();

                let result = match func::req(webpath.as_str(), &endpoint).await {
                    Ok(res) => res,
                    Err(_) => format!("{}{}: STATUS_CODE = error", webpath, endpoint),
                };

                println!("{}", result);

                drop(permit);
                let completed = progress.fetch_add(1, Ordering::SeqCst) + 1;

                print!("\rProgress: {}/{} | ", completed, total);
                io::stdout().flush().unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await?;
        }
        println!("\nAll requests completed.");
    } else {
        let res = func::req(&webpath, &endpoint_input).await?;
        println!("{}", res);
    }

    Ok(())
}