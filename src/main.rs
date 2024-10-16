
mod func;
mod args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (webpath, endpoint_input, user_agent, threads, proxy) = args::parse_args();
    let _ = func::process(webpath, endpoint_input, user_agent, threads, proxy).await;

    Ok(())
}