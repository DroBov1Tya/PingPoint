mod func;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (webpath, endpoint_input, user_agent, threads) = func::parse_args();
    let _ = func::process(webpath, endpoint_input, user_agent, threads).await;

    Ok(())
}