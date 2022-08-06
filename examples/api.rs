use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cid = "bafybeibqw5s44zssunqpgtde7tkoeotd3jcnucv4groto4gfjejvks6wdy";

    let status = w3s::api::status_of_cid(cid).await?;
    let head = w3s::api::check_car_head(cid).await?;
    let car = w3s::api::retrieve_car(cid).await?;

    println!("status: {:?}", status);
    println!("head: {:?}", head);
    println!("car file length: {}", car.len());
    
    Ok(())
}