use super::*;

#[test]
fn deserialize_verifier_lookup_request() {
    /*
    curl -X POST \
         -H 'Content-Type: application/json' \
         -d '{"jsonrpc":"2.0","id":10,"method":"VerifierLookupRequest","params":{"verifier":"partisia-twitter-mainnet", "verifier_id":"twitter|1415723267256639488"}}' \
         https://torus-19.torusnode.com/jrpc | jq

     */
    let res_json = r#"{
      "jsonrpc": "2.0",
      "result": {
        "keys": [
          {
            "key_index": "14745a",
            "pub_key_X": "436676f1c06a11f805a92d5d02a5789296c562d1aeb8e72d6318760f61cdcbf",
            "pub_key_Y": "afd563755d627d1ae4021d60863acca0c3bf4e5d8f5ce24c91e55ebbf5b263b0",
            "address": "0xC9F0af3d1D6089992C0041902D846c4b448311F2"
          }
        ]
      },
      "id": 10
    }"#;
    let json: JsonRpc<TorusKeys> = serde_json::from_str(res_json).unwrap();
    assert_eq!(
        json.result.keys[0].pub_key_x,
        "436676f1c06a11f805a92d5d02a5789296c562d1aeb8e72d6318760f61cdcbf"
    );
}
#[test]
fn deserialize_key_lookup_request() {
    /*
    curl -X POST \
         -H 'Content-Type: application/json' \
         -d '{"jsonrpc":"2.0","id":10,"method":"KeyLookupRequest","params":{"pub_key_X":"436676f1c06a11f805a92d5d02a5789296c562d1aeb8e72d6318760f61cdcbf","pub_key_Y":"afd563755d627d1ae4021d60863acca0c3bf4e5d8f5ce24c91e55ebbf5b263b0"}}' \
         https://torus-19.torusnode.com/jrpc | jq
    */
    let res_json = r#"{
      "jsonrpc": "2.0",
      "result": {
        "Index": "14745a",
        "PublicKey": {
          "X": "436676f1c06a11f805a92d5d02a5789296c562d1aeb8e72d6318760f61cdcbf",
          "Y": "afd563755d627d1ae4021d60863acca0c3bf4e5d8f5ce24c91e55ebbf5b263b0"
        },
        "Threshold": 1,
        "Verifiers": {
          "partisia-twitter-mainnet": [
            "twitter|1415723267256639488"
          ]
        }
      },
      "id": 10
    }"#;
    let json: JsonRpc<TorusLookup> = serde_json::from_str(res_json).unwrap();
    assert_eq!(
        json.result.public_key.x,
        "436676f1c06a11f805a92d5d02a5789296c562d1aeb8e72d6318760f61cdcbf"
    );
    assert_eq!(
        json.result.public_key.y,
        "afd563755d627d1ae4021d60863acca0c3bf4e5d8f5ce24c91e55ebbf5b263b0"
    );

    let ary_ids = json.result.verifiers.partisia.unwrap();
    assert_eq!(ary_ids.len(), 1);
    assert_eq!(ary_ids[0], "twitter|1415723267256639488");
    assert_eq!(
        ary_ids[0].splitn(2, "|").collect::<Vec<&str>>()[1],
        "1415723267256639488"
    );
}

#[test]
fn num() {
    assert_eq!(9 / 2, 4);
    assert_eq!(TORUS_ENDPOINTS.len() / 2, 4);
    assert_eq!(TORUS_ENDPOINTS.len() / 2 + 1, 5);
}

#[tokio::test]
async fn rpc_fetch_consensus() {
    let j = json!({"jsonrpc":"2.0","id":10,"method":"VerifierLookupRequest","params":{"verifier":"partisia-twitter-mainnet", "verifier_id":"twitter|1415723267256639488"}});
    let x: TorusKeys = consensus_multi_thread::rpc_with_consensus(&j)
        .await
        .unwrap();
    assert_eq!(x.keys.len(), 1);
    assert_eq!(
        x.keys[0].pub_key_x,
        "436676f1c06a11f805a92d5d02a5789296c562d1aeb8e72d6318760f61cdcbf"
    );
    assert_eq!(
        x.keys[0].pub_key_y,
        "afd563755d627d1ae4021d60863acca0c3bf4e5d8f5ce24c91e55ebbf5b263b0"
    );
}

#[tokio::test]
async fn id_lookup() {
    let torus_key = multi_thread::lookup_request("twitter|1415723267256639488", Verifier::Twitter)
        .await
        .unwrap();

    assert!(torus_key.is_some());
    assert_eq!(torus_key, Some(hex_literal::hex!("040436676f1c06a11f805a92d5d02a5789296c562d1aeb8e72d6318760f61cdcbfafd563755d627d1ae4021d60863acca0c3bf4e5d8f5ce24c91e55ebbf5b263b0")));
    // assert_eq!(
    //     torus_keys.keys[0].pub_key_x,
    //     // Note that the hex string does not have the leading zero
    //     "436676f1c06a11f805a92d5d02a5789296c562d1aeb8e72d6318760f61cdcbf"
    // );
    // assert_eq!(
    //     torus_keys.keys[0].pub_key_y,
    //     "afd563755d627d1ae4021d60863acca0c3bf4e5d8f5ce24c91e55ebbf5b263b0"
    // );
}
#[tokio::test]
async fn id_lookup_discord() {
    let torus_key = multi_thread::lookup_request("783831719589314610", Verifier::Discord)
        .await
        .unwrap();

    assert_eq!(torus_key, Some(hex_literal::hex!("0460026de34a9104b819e477bbee95256673f288fe6c116515aa95af1a046a26df169549acdf77781a2a390bb2379dee7f02aa3a016f5279f65be7b42f016ba445")));
    // assert_eq!(
    //     torus_keys.keys[0].pub_key_x,
    //     // Note that the hex string does not have the leading zero
    //     "60026de34a9104b819e477bbee95256673f288fe6c116515aa95af1a046a26df"
    // );
    // assert_eq!(
    //     torus_keys.keys[0].pub_key_y,
    //     "169549acdf77781a2a390bb2379dee7f02aa3a016f5279f65be7b42f016ba445"
    // );
}

// #[tokio::test]
// async fn key_lookup() {
//     let x = hex_literal::hex!("0436676f1c06a11f805a92d5d02a5789296c562d1aeb8e72d6318760f61cdcbf");
//     let y = hex_literal::hex!("afd563755d627d1ae4021d60863acca0c3bf4e5d8f5ce24c91e55ebbf5b263b0");
//     let id = multi_thread::key_lookup_request(&x, &y).await.unwrap();
//     assert_eq!(id, Some(1415723267256639488));
// }

// #[tokio::test]
// async fn key_lookup_not_found() {
//     /*
//     "Verifiers": {
//       "google": [
//         "leonard@tor.us"
//       ]
//     } */
//     let x = hex_literal::hex!("1ac083ce3e501588a9cae005473074aaad13897185112dc84b80b0dbe691c237");
//     let y = hex_literal::hex!("19a61da30fd14f83d3ccefe8ad6ff15d41fb55133baa895d85c08aa518827789");
//     let key = multi_thread::key_lookup_request(&x, &y).await.unwrap();
//     assert_eq!(key, None);
// }
