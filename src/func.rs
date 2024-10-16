use reqwest::{Client, Proxy};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use tokio::sync::Semaphore;
use tokio::task;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use crossterm::{cursor, terminal, ExecutableCommand};
use std::{fs, io::{self, stdout, BufRead, BufReader, Write}, sync::{atomic::{AtomicUsize, Ordering}, Arc}};

mod user_agents;
mod art;

async fn req(webpath: &str, endpoint: &str, random_agent: bool, proxy: Option<String>) -> Result<String, Box<dyn std::error::Error  + Send + Sync>> {
    let client = if let Some(proxy_url) = proxy {
        Client::builder()
            .proxy(Proxy::all(&proxy_url)?)
            .danger_accept_invalid_certs(true)
            .build()?
    } else {
        Client::new()
    };

    let mut request = client.get(&format!("{}{}", webpath, endpoint));

    let mut rng = StdRng::from_entropy();
    let user_agent = read_user_agents().await;
    let random = rng.gen_range(0..=user_agent.len() - 1);

    if random_agent {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&user_agent[random])?);
        request = request.headers(headers);
    }
    
    let result = request.send().await?;

    let status_code = result.status();
    let result_str: String = format!("
    URL:    {}{}\n
    STATUS: {}\n", 
    webpath, endpoint, status_code);

    Ok(result_str)
}

pub async fn read_user_agents() -> [String; 1000] {
    let mut user_agents_array: [String; 1000] = std::array::from_fn(|_| String::new());

    let user_agents_iter = user_agents::USER_AGENTS
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string());

    // Заполняем массив строками из итератора
    for (i, user_agent) in user_agents_iter.enumerate() {
        if i < 1000 {
            user_agents_array[i] = user_agent;
        } else {
            break;
        }
    }

    user_agents_array
}

async fn clear_output(message: Option<&str>, stage: bool) {
    match stage{
        true => {
            stdout().execute(terminal::Clear(terminal::ClearType::CurrentLine)).unwrap();
            println!("{}", message.unwrap());
            
            stdout().execute(cursor::MoveDown(1)).unwrap();
            io::stdout().flush().unwrap();
        }
        false => {
            let _ = stdout().execute(terminal::Clear(terminal::ClearType::CurrentLine));
            println!("\nAll requests completed.");
            let _ = stdout().execute(cursor::Show);
        }
    }
}

pub async fn process(
    webpath:String, 
    endpoint: String, 
    random_agent: bool, 
    threads: usize,
    proxy: Option<String>) 
    -> Result<(), Box<dyn std::error::Error>>{

    stdout().execute(terminal::Clear(terminal::ClearType::All))?;
    println!("{}\n", art::ART);
    stdout().execute(cursor::Hide)?;

    if let Ok(file) = fs::File::open(&endpoint) {
        let reader = BufReader::new(file);

        let webpath = Arc::new(webpath);
        let random_agent = Arc::new(random_agent);
        let proxy = Arc::new(proxy);
        let endpoints: Vec<_> = reader.lines().collect::<Result<_, _>>()?;
        
        let progress = Arc::new(AtomicUsize::new(0));
        let total = endpoints.len();

        let semaphore = Arc::new(Semaphore::new(threads));
        let mut handles = Vec::new();

        for endpoint in endpoints {
            let random_agent = *random_agent;
            let webpath = Arc::clone(&webpath);
            let proxy = Arc::clone(&proxy);
            let progress = Arc::clone(&progress);
            let semaphore = Arc::clone(&semaphore);

            let handle = task::spawn(async move {
                let permit = semaphore.acquire_owned().await.unwrap();
                let result;

                if let Some(proxy) = proxy.as_deref() {
                    result = req(&webpath, &endpoint, random_agent, Some(proxy.to_string())).await;
                } else {
                    result = req(&webpath, &endpoint, random_agent, None).await;
                }
                let output = match &result {
                    Ok(res) => res.clone(),
                    Err(e) => format!("Error: {}", e),
                };
                println!("-----------------------------------------");
                clear_output(Some(&output), true).await;

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
        clear_output(None, false).await;
        stdout().execute(cursor::MoveDown(1)).unwrap();
    } else {
        if proxy.is_some(){
            match req(webpath.as_str(), &endpoint, random_agent, proxy).await {
                Ok(res) => println!("{}", res),
                Err(e) => println!("Error: {}\n URL: {}{}", webpath.as_str(), &endpoint, e),
        }
        }
    }
    Ok(())
}