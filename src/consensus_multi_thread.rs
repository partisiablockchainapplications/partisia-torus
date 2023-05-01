use super::*;

#[derive(Debug, Deserialize, Serialize)]
struct FoldGroups<'a> {
    key: [u8; 32],
    result: &'a [u8],
    count: usize,
}

async fn call_endpoint<T>(json_rpc: &Value, endpoint: &str) -> Result<Vec<u8>>
where
    for<'de> T: Deserialize<'de>,
    T: Serialize,
    T: std::fmt::Debug,
{
    let client = Client::new();
    let mut header_map = HeaderMap::new();
    header_map.insert(header::CONTENT_TYPE, "json".parse()?);

    let res = client
        .post(endpoint)
        .timeout(Duration::from_millis(3000))
        .headers(header_map.clone())
        .json(json_rpc)
        .send()
        .await?;

    ensure!(res.status().is_success());
    let v = res.json::<JsonRpc<T>>().await?;

    let ser = bincode::serialize(&v.result)?;
    Ok(ser)
}

async fn handle_jsonrpc_request<T>(
    json_rpc: &Value,
    endpoint: &str,
    map: MapRpcResultsMultiThread<ConsensusResults>,
    idx: usize,
) -> Result<T>
where
    for<'de> T: Deserialize<'de>,
    T: Serialize,
    T: std::fmt::Debug,
{
    // call endpoint and update the shared map with the result
    match call_endpoint::<T>(json_rpc, endpoint).await {
        Ok(ser) => map.write().await[idx] = Some(Ok(ser)),
        Err(e) => map.write().await[idx] = Some(Err(e)),
    };

    // take the map and check each for consensus with greater than 50%
    let x = &*map.read().await;
    let (completed, _pending): (Vec<_>, Vec<_>) =
        x.iter().map(|x| x.as_ref()).partition(Option::is_some);
    let (results, errors): (Vec<_>, Vec<_>) = completed
        .into_iter()
        .map(|x| x.unwrap().as_ref())
        .partition(Result::is_ok);

    // more than 50% have completed with results
    let consensus_num = TORUS_ENDPOINTS.len() / 2 + 1;

    if results.len() >= consensus_num {
        // group the matches and count how many are the same using sha256 hash
        let mut res_grouped = results.into_iter().map(Result::unwrap).fold(
            Vec::new(),
            |mut acc: Vec<FoldGroups>, buf| {
                let hash_key = sha256_hash(buf);
                let f = acc.iter().position(|g| g.key == hash_key);
                match f {
                    Some(idx) => acc[idx].count += 1,
                    None => acc.push(FoldGroups {
                        key: hash_key,
                        result: buf,
                        count: 1,
                    }),
                };
                acc
            },
        );
        // sort result by count desc
        res_grouped.sort_by(|a, b| b.count.cmp(&a.count));

        // take the group with highest count
        let group = res_grouped.remove(0);

        // check consensus threshold is met and return the result
        if group.count >= consensus_num {
            // at this point we have reach consensus so we can safely return early without needing any other endpoints to finish
            Ok(bincode::deserialize(group.result)?)
        } else {
            // we have enough results to form a consensus but not enough agree with each other
            bail!("no consensus");
        }
    } else {
        if results.len() + errors.len() == TORUS_ENDPOINTS.len() {
            let err = errors[0].unwrap_err().to_string();
            bail!(err);
        } else {
            // we do not have enough successful results
            bail!("pending more results");
        }
    }
}

pub async fn rpc_with_consensus<T>(json_value: &Value) -> Result<T>
where
    for<'de> T: Deserialize<'de>,
    T: Serialize,
    T: std::fmt::Debug,
{
    let init: ConsensusResults = TORUS_ENDPOINTS
        .iter()
        .map(|_| None)
        .collect::<Vec<Option<Result<Vec<u8>>>>>()
        .try_into()
        .unwrap();

    let map: MapRpcResultsMultiThread<ConsensusResults> = Arc::new(RwLock::new(init));
    let vec_futures: Vec<_> = TORUS_ENDPOINTS
        .into_iter()
        .enumerate()
        .map(|(i, s)| Box::pin(handle_jsonrpc_request(json_value, s, Arc::clone(&map), i)))
        .collect();

    let (res, _) = futures::future::select_ok(vec_futures).await?;
    Ok(res)
}
