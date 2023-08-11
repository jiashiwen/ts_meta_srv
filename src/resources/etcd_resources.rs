use crate::configure::get_config;
use anyhow::Result;
use etcd_client::Client;

// 初始化 etcd 数据源
pub async fn init_etcd_client() -> Result<Client> {
    let config = get_config()?;
    println!("{:?}",config.etcd);
    let client = Client::connect(config.etcd.endpoints, None).await?;
    Ok(client)
}
