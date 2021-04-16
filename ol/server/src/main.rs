use ol_cli::{account, chain_info, check_runner, node_health};
use futures::StreamExt;
use std::convert::Infallible;
use std::thread;
use std::time::Duration;
use std::{fs, path::PathBuf};
use tokio::{time::interval};
use warp::{sse::ServerSentEvent, Filter};
use serde_json::json;
use cli::libra_client::LibraClient;
use libra_json_rpc_client::AccountAddress;
use ol_cli::{client::pick_client, config::OlCliConfig};
use toml;
const DEFAULT_CONFIG_PATH: &str = "/root/.0L/0L.toml";

fn main() {
    // TODO: fetch swarm configs from command line.
    // TODO: fetch optional config path
    let cfg = parse_configs(None);
    let client = pick_client(None, &cfg);
    start_server(client, cfg.profile.account);
}

fn sse_check(info: node_health::Items) -> Result<impl ServerSentEvent, Infallible> {
    Ok(warp::sse::json(info))
}

fn sse_chain_info(info: chain_info::ChainInfo) -> Result<impl ServerSentEvent, Infallible> {
    Ok(warp::sse::json(info))
}

fn sse_val_info(info: Vec<chain_info::ValidatorInfo>) -> Result<impl ServerSentEvent, Infallible> {
    Ok(warp::sse::json(info))
}

fn sse_account_info(info: account::AccountInfo) -> Result<impl ServerSentEvent, Infallible> {
    Ok(warp::sse::json(info))
}

#[tokio::main]
pub async fn start_server(client: LibraClient, address: AccountAddress) {
    // TODO: Perhaps a better way to keep the check cache fresh?
    thread::spawn(move || {
        check_runner::mon(client, address, true, false);
    });

    //GET check/ (json api for check data)
    let check = warp::path("check").and(warp::get()).map(|| {
        // let mut health = node_health::NodeHealth::new();
        // create server event source from Check object
        let event_stream = interval(Duration::from_secs(10)).map(move |_| {
            let items = node_health::Items::read_cache().unwrap();
            // let items = health.refresh_checks();
            sse_check(items)
        });
        // reply using server-sent events
        warp::sse::reply(event_stream)
    });

    //GET chain/ (the json api)
    let chain_live = warp::path("chain_live").and(warp::get()).map(|| {
        // create server event source
        let event_stream = interval(Duration::from_secs(10)).map(move |_| {
            let info = crate::chain_info::read_chain_info_cache();
            sse_chain_info(info)
        });
        // reply using server-sent events
        warp::sse::reply(event_stream)
    });

    // let configs = warp::path("static")
    // .and(warp::fs::file("/root/.0L/genesis_waypoint"));
    let vals = warp::path("vals")
    .and(warp::get()
    .map(|| { 
      let vals = crate::chain_info::read_val_info_cache();
      warp::reply::json(&vals)
     }));

    let chain = warp::path("chain")
    .and(warp::get()
    .map(|| { 
      let chain = crate::chain_info::read_chain_info_cache();
      warp::reply::json(&chain)
     }));

    let account_template = warp::path("account.json")
    .and(warp::get()
    .map(|| { 
      fs::read_to_string("/root/.0L/account.json").unwrap()
      // let obj: Value = serde_json::from_str(&string);
      
     }));

    let epoch = warp::path("epoch.json")
    .and(warp::get()
    .map(|| { 
      let ci = crate::chain_info::read_chain_info_cache();
      let json = json!({
        "epoch": ci.epoch,
        "waypoint": ci.waypoint.unwrap().to_string()
      });
      json.to_string()
    }));


    //GET account/ (the json api)
    let account = warp::path("account").and(warp::get()).map(|| {
        // create server event source
        let event_stream = interval(Duration::from_secs(60)).map(move |_| {
            let info = crate::account::AccountInfo::read_account_info_cache();
            sse_account_info(info)
        });
        // reply using server-sent events
        warp::sse::reply(event_stream)
    });


    //GET validators/ (the json api)
    let vals_live = warp::path("validators").and(warp::get()).map(|| {
        // create server event source
        let event_stream = interval(Duration::from_secs(60)).map(move |_| {
            let info = crate::chain_info::read_val_info_cache();
            // TODO: Use a different data source for /explorer/ data.
            sse_val_info(info)
        });
        // reply using server-sent events
        warp::sse::reply(event_stream)
    });

    //GET /
    let home = warp::fs::dir("/root/libra/ol-cli/web-monitor/public/");

    warp::serve(home.or(check).or(chain).or(chain_live).or(vals_live).or(vals).or(account_template).or(epoch).or(account))
        .run(([0, 0, 0, 0], 3030)).await;
}


pub fn parse_configs(path_opt: Option<PathBuf>) -> OlCliConfig {
  
  let toml_path = if path_opt.is_some() {
    path_opt.unwrap()
  } else {
    PathBuf::from(DEFAULT_CONFIG_PATH)
  };

  let str = fs::read_to_string(&toml_path).expect(&format!("could not open file: {:?}", &toml_path));
  let cfg: OlCliConfig = toml::from_str(&str).unwrap();
  cfg
}