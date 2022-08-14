use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();

    match args.as_slice() {
        [_, auth_token] => get_uploads(auth_token).await,
        _ => panic!(
            "\n\nPlease input the [web3.storage_auth_token(eyJhbG......MHlq0)]\n\n"
        ),
    }
}

async fn get_uploads(auth_token: &String) -> Result<()> {
    let query = w3s::api::UserUploadsQuery::new(None, None, None, None, None);
    let results = w3s::api::fetch_uploads(auth_token, query).await?;
    println!("cid list: {:?}", results.into_iter().map(|x| x.cid).collect::<Vec<_>>());

    Ok(())
}
