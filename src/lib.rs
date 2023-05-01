use anyhow::{bail, ensure, Result};
use reqwest::{
    header::{self, HeaderMap},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};
use tokio::sync::RwLock;

mod consensus_multi_thread;
mod consensus_single_thread;
#[cfg(test)]
mod tests;

// https://github.com/torusresearch/fetch-node-details/blob/master/src/nodeDetailManager.ts
const TORUS_ENDPOINTS: [&'static str; 9] = [
    "https://torus-19.torusnode.com/jrpc",
    "https://torus-node.ens.domains/jrpc",
    "https://torus-node.matic.network/jrpc",
    "https://torus.zilliqa.network/jrpc",
    "https://torus-mainnet.cosmos.network/jrpc",
    "https://torus2.etherscan.com/jrpc",
    "https://torus-node-v2.skalelabs.com/jrpc",
    "https://torus-node.binancex.dev/jrpc",
    "https://torusnode.ont.io/jrpc",
];

const VERIFIER_TWITTER: &'static str = "partisia-twitter-mainnet";
const VERIFIER_DISCORD: &'static str = "partisia-discord";

// the consensus results are None if still pending a result from the rpc call
type ConsensusResults = [Option<Result<Vec<u8>>>; TORUS_ENDPOINTS.len()];
type MapRpcResultsSingleThread<T> = Rc<RefCell<T>>;
type MapRpcResultsMultiThread<T> = Arc<RwLock<T>>;

fn sha256_hash(buf: &[u8]) -> [u8; 32] {
    // get the hash
    let mut hasher = Sha256::new();
    hasher.update(buf);
    hasher
        .finalize()
        .try_into()
        .expect("sha256 is always 32 bytes")
}
#[derive(Debug, Deserialize)]
struct JsonRpc<T> {
    // jsonrpc: String,
    result: T,
    // id: u8,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TorusKeys {
    keys: Vec<TorusKey>,
}
#[derive(Debug, Deserialize, Serialize)]
struct TorusKey {
    key_index: String,
    #[serde(rename = "pub_key_X")]
    pub_key_x: String,
    #[serde(rename = "pub_key_Y")]
    pub_key_y: String,
    address: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TorusLookup {
    #[serde(rename = "Index")]
    index: String,
    #[serde(rename = "PublicKey")]
    public_key: TorusPublicKey,
    #[serde(rename = "Threshold")]
    threshold: u16,
    #[serde(rename = "Verifiers")]
    verifiers: TorusVerifier,
}

#[derive(Debug, Deserialize, Serialize)]
struct TorusVerifier {
    #[serde(rename = "partisia-twitter-mainnet")]
    partisia: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TorusPublicKey {
    #[serde(rename = "X")]
    x: String,
    #[serde(rename = "Y")]
    y: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Verifier {
    Twitter,
    Discord,
}

#[cfg(feature = "multi_thread")]
pub mod multi_thread {
    use super::*;

    pub async fn lookup_request(verifier_id: u64, verifier_type: Verifier) -> Result<TorusKeys> {
        let verifier = match verifier_type {
            Verifier::Twitter => VERIFIER_TWITTER,
            Verifier::Discord => VERIFIER_DISCORD,
        };
        let json_rpc = json!({
          "jsonrpc": "2.0",
          "id": 10,
          "method": "VerifierLookupRequest",
          "params": {
            "verifier": verifier,
            "verifier_id": verifier_id
          }
        });

        Ok(consensus_multi_thread::rpc_with_consensus(&json_rpc).await?)
    }
    pub async fn key_lookup_request(
        pub_key_x: &[u8; 32],
        pub_key_y: &[u8; 32],
    ) -> Result<Option<u64>> {
        let json_rpc = json!({
          "jsonrpc": "2.0",
          "id": 10,
          "method": "KeyLookupRequest",
          "params": {
            "pub_key_X": hex::encode(pub_key_x),
            "pub_key_Y": hex::encode(pub_key_y)
          }
        });

        let torus_lookup: TorusLookup =
            consensus_multi_thread::rpc_with_consensus(&json_rpc).await?;
        if let Some(ary_ids) = torus_lookup.verifiers.partisia {
            ensure!(ary_ids.len() > 0, "No id found for partisia");

            // take the last key which will be formatted like "twitter|1415723267256639488" and split it
            let twitter_id = &ary_ids[ary_ids.len() - 1]
                .splitn(2, "|")
                .collect::<Vec<&str>>();
            ensure!(twitter_id.len() == 2, "malformed twitter key");
            Ok(Some(twitter_id[1].parse()?))
        } else {
            // No key found for partisia
            Ok(None)
        }
    }
}

#[cfg(feature = "single_threaded")]
pub mod single_threaded {
    use super::*;

    pub async fn lookup_request(verifier_id: u64, verifier_type: Verifier) -> Result<TorusKeys> {
        let verifier = match verifier_type {
            Verifier::Twitter => VERIFIER_TWITTER,
            Verifier::Discord => VERIFIER_DISCORD,
        };
        let json_rpc = json!({
          "jsonrpc": "2.0",
          "id": 10,
          "method": "VerifierLookupRequest",
          "params": {
            "verifier": verifier,
            "verifier_id": verifier_id
          }
        });

        Ok(consensus_single_thread::rpc_with_consensus(&json_rpc).await?)
    }
    pub async fn key_lookup_request(
        pub_key_x: &[u8; 32],
        pub_key_y: &[u8; 32],
    ) -> Result<Option<u64>> {
        let json_rpc = json!({
          "jsonrpc": "2.0",
          "id": 10,
          "method": "KeyLookupRequest",
          "params": {
            "pub_key_X": hex::encode(pub_key_x),
            "pub_key_Y": hex::encode(pub_key_y)
          }
        });

        let torus_lookup: TorusLookup =
            consensus_single_thread::rpc_with_consensus(&json_rpc).await?;
        if let Some(ary_ids) = torus_lookup.verifiers.partisia {
            ensure!(ary_ids.len() > 0, "No id found for partisia");

            // take the last key which will be formatted like "twitter|1415723267256639488" and split it
            let twitter_id = &ary_ids[ary_ids.len() - 1]
                .splitn(2, "|")
                .collect::<Vec<&str>>();
            ensure!(twitter_id.len() == 2, "malformed twitter key");
            Ok(Some(twitter_id[1].parse()?))
        } else {
            // No key found for partisia
            Ok(None)
        }
    }
}
