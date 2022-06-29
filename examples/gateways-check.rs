#[tokio::main]
async fn main() {
    let gateways = w3s::gateway::check_gateways(None).await;
    println!("{:#?}", gateways);
}
