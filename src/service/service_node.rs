use crate::{
    commons::json_to_struct,
    httpserver::module::{NodeInfo, NODE_ID_PREFIX},
    resources::GLOBAL_ETCD,
};
use anyhow::Result;
use etcd_client::GetOptions;

pub async fn nodes_info() -> Result<Vec<NodeInfo>> {
    let mut vec_nodes: Vec<NodeInfo> = vec![];
    unsafe {
        match GLOBAL_ETCD.get_mut() {
            Some(etcd_clinet) => {
                let resp = etcd_clinet
                    .get(NODE_ID_PREFIX, Some(GetOptions::new().with_prefix()))
                    .await?;

                for kv in resp.kvs() {
                    let vec_to_string = String::from_utf8(kv.value().to_vec())?;
                    let nodeinfo = json_to_struct::<NodeInfo>(&vec_to_string)?;
                    vec_nodes.push(nodeinfo);
                }
            }
            None => {}
        }
    }
    Ok(vec_nodes)
}
