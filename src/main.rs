

mod func;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let webpath: String = "https://tourofrust.com/".to_string();
    let endpoint: String = "96_ru.html".to_string();


    let res = func::request::req(webpath, endpoint).await?;
    println!("{}", res);

    Ok(())
}