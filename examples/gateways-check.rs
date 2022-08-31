use std::env;

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<_>>();

    match args.as_slice() {
        [_, cid, time_out_sec, proxy] => {
            let gateways = w3s::gateway::check_gateways_by_cid(
                cid,
                time_out_sec.parse::<u64>().ok(),
                Some(proxy),
            )
            .await;
            println!("{:#?}", gateways);
        }
        [_, cid, time_out_sec] => {
            let gateways =
                w3s::gateway::check_gateways_by_cid(cid, time_out_sec.parse::<u64>().ok(), None)
                    .await;
            println!("{:#?}", gateways);
        }
        _ => {
            let gateways = w3s::gateway::check_gateways(None).await;
            println!("{:#?}", gateways);
        }
    }
}
