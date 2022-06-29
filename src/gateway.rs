//! IPFS gateway utilities
//! 

use reqwest::Client;
use futures::{future, future::select_all, FutureExt, TryFutureExt};
use std::time::{Duration, Instant};

/// Checks all the public gateways and returns the available ones with response time.
/// 
/// This function is inspired from [public-gateway-checker](https://ipfs.github.io/public-gateway-checker/).
/// * `custom_timeout_secs`: Specify the custom timeout seconds for a check. The default timeout seconds is 3 seconds.
pub async fn check_gateways(custom_timeout_secs: Option<u64>) -> Vec<(&'static str, f32)> {
    let mut futures = GATEWAYS
        .iter()
        .enumerate()
        .map(|(index, gateway)| {
            let client = Client::builder()
                .timeout(Duration::from_secs(custom_timeout_secs.unwrap_or(3)))
                .build()
                .unwrap_or(Client::new());

            let t = Instant::now();

            client
                .get(format!("{}{}", gateway, GATEWAY_CHECKER_CID))
                .send()
                .and_then(move |resp| resp.text().and_then(move |x| future::ok((index, x, t))))
                .map_err(move |_| index)
                .boxed()
        })
        .collect::<Vec<_>>();

    let mut ret = vec![];
    while futures.len() > 0 {
        let (result, _, remaining_futures) = select_all(futures).await;
        futures = remaining_futures;

        if let Ok((index, content, t)) = result {
            if content.trim() == GATEWAY_CHECK_WORDS {
                let gateway = GATEWAYS[index];
                let delta_t = Instant::now() - t;
                ret.push((gateway, delta_t.as_secs_f32()));
            }
        }
    }

    ret
}

/// The gateway data source: [gateways.json](https://github.com/ipfs/public-gateway-checker/blob/master/src/gateways.json)
static GATEWAYS: [&'static str; 95] = [
    "https://ipfs.io/ipfs/",
    "https://dweb.link/ipfs/",
    "https://gateway.ipfs.io/ipfs/",
    "https://ipfs.infura.io/ipfs/",
    "https://infura-ipfs.io/ipfs/",
    "https://ninetailed.ninja/ipfs/",
    "https://via0.com/ipfs/",
    "https://ipfs.eternum.io/ipfs/",
    "https://hardbin.com/ipfs/",
    "https://gateway.blocksec.com/ipfs/",
    "https://cloudflare-ipfs.com/ipfs/",
    "https://astyanax.io/ipfs/",
    "https://cf-ipfs.com/ipfs/",
    "https://ipns.co/ipfs/",
    "https://ipfs.mrh.io/ipfs/",
    "https://gateway.originprotocol.com/ipfs/",
    "https://gateway.pinata.cloud/ipfs/",
    "https://ipfs.doolta.com/ipfs/",
    "https://ipfs.sloppyta.co/ipfs/",
    "https://ipfs.busy.org/ipfs/",
    "https://ipfs.greyh.at/ipfs/",
    "https://gateway.serph.network/ipfs/",
    "https://jorropo.net/ipfs/",
    "https://ipfs.fooock.com/ipfs/",
    "https://cdn.cwinfo.net/ipfs/",
    "https://aragon.ventures/ipfs/",
    "https://ipfs-cdn.aragon.ventures/ipfs/",
    "https://permaweb.io/ipfs/",
    "https://ipfs.best-practice.se/ipfs/",
    "https://storjipfs-gateway.com/ipfs/",
    "https://ipfs.runfission.com/ipfs/",
    "https://ipfs.trusti.id/ipfs/",
    "https://ipfs.overpi.com/ipfs/",
    "https://gateway.ipfs.lc/ipfs/",
    "https://ipfs.ink/ipfs/",
    "https://ipfsgateway.makersplace.com/ipfs/",
    "https://gateway.ravenland.org/ipfs/",
    "https://ipfs.funnychain.co/ipfs/",
    "https://ipfs.telos.miami/ipfs/",
    "https://ipfs.mttk.net/ipfs/",
    "https://ipfs.fleek.co/ipfs/",
    "https://ipfs.jbb.one/ipfs/",
    "https://ipfs.yt/ipfs/",
    "https://hashnews.k1ic.com/ipfs/",
    "https://ipfs.vip/ipfs/",
    "https://ipfs.drink.cafe/ipfs/",
    "https://ipfs.azurewebsites.net/ipfs/",
    "https://gw.ipfspin.com/ipfs/",
    "https://ipfs.kavin.rocks/ipfs/",
    "https://ipfs.denarius.io/ipfs/",
    "https://ipfs.mihir.ch/ipfs/",
    "https://crustwebsites.net/ipfs/",
    "https://ipfs0.sjc.cloudsigma.com/ipfs/",
    "http://ipfs.genenetwork.org/ipfs/",
    "https://ipfs.eth.aragon.network/ipfs/",
    "https://ipfs.smartholdem.io/ipfs/",
    "https://ipfs.xoqq.ch/ipfs/",
    "http://natoboram.mynetgear.com:8080/ipfs/",
    "https://video.oneloveipfs.com/ipfs/",
    "http://ipfs.anonymize.com/ipfs/",
    "https://ipfs.taxi/ipfs/",
    "https://ipfs.scalaproject.io/ipfs/",
    "https://search.ipfsgate.com/ipfs/",
    "https://ipfs.decoo.io/ipfs/",
    "https://ivoputzer.xyz/ipfs/",
    "https://alexdav.id/ipfs/",
    "https://ipfs.uploads.nu/ipfs/",
    "https://hub.textile.io/ipfs/",
    "https://ipfs1.pixura.io/ipfs/",
    "https://ravencoinipfs-gateway.com/ipfs/",
    "https://konubinix.eu/ipfs/",
    "https://3cloud.ee/ipfs/",
    "https://ipfs.tubby.cloud/ipfs/",
    "https://ipfs.lain.la/ipfs/",
    "https://ipfs.adatools.io/ipfs/",
    "https://ipfs.kaleido.art/ipfs/",
    "https://ipfs.slang.cx/ipfs/",
    "https://ipfs.arching-kaos.com/ipfs/",
    "https://storry.tv/ipfs/",
    "https://ipfs.kxv.io/ipfs/",
    "https://ipfs.1-2.dev/ipfs/",
    "https://ipfs-nosub.stibarc.com/ipfs/",
    "https://dweb.eu.org/ipfs/",
    "https://permaweb.eu.org/ipfs/",
    "https://ipfs.namebase.io/ipfs/",
    "https://ipfs.tribecap.co/ipfs/",
    "https://ipfs.kinematiks.com/ipfs/",
    "https://c4rex.co/ipfs/",
    "https://nftstorage.link/ipfs/",
    "https://gravity.jup.io/ipfs/",
    "http://fzdqwfb5ml56oadins5jpuhe6ki6bk33umri35p5kt2tue4fpws5efid.onion/ipfs/",
    "https://tth-ipfs.com/ipfs/",
    "https://ipfs.chisdealhd.co.uk/ipfs/",
    "https://ipfs.alloyxuast.tk/ipfs/",
    "https://ipfs.litnet.work/ipfs/",
];

const GATEWAY_CHECKER_CID: &'static str =
    "bafybeifx7yeb55armcsxwwitkymga5xf53dxiarykms3ygqic223w5sk3m";

const GATEWAY_CHECK_WORDS: &'static str = "Hello from IPFS Gateway Checker";
