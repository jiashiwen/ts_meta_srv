use anyhow::Result;
use etcd_client::Client;

use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use rbatis::Rbatis;
use std::sync::Mutex;

use super::init_etcd_client;
use super::mysql_rbatis_resource::init_datasource_mysql;
use super::redis_resource::gen_redis_conn_pool;
use super::redis_resource::RedisConnectionManager;

// 全局 redis pool
pub static GLOBAL_REDIS_POOL: OnceCell<r2d2::Pool<RedisConnectionManager>> = OnceCell::new();
pub static mut GLOBAL_ETCD: OnceCell<Client> = OnceCell::new();

pub static GLOBAL_RBATIS_MYSQL: Lazy<rbatis::Rbatis> = Lazy::new(|| {
    let rb = match init_datasource_mysql() {
        Ok(rb) => rb,
        Err(err) => panic!("{}", err),
    };
    rb
});

static GLOBAL_PD_ENDPOINT: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

pub struct DataSourceMysql {
    pub rbatis: Rbatis,
}

pub async fn init_resources() -> Result<()> {
    // let cfg = get_config()?;
    // init_global_redis();
    // init_global_rbatis_mysql().await?;
    init_global_etcd_client().await?;
    Ok(())
}

fn init_global_redis() {
    GLOBAL_REDIS_POOL.get_or_init(|| {
        let pool = match gen_redis_conn_pool() {
            Ok(it) => it,
            Err(err) => panic!("{}", err.to_string()),
        };
        pool
    });
}

async fn init_global_rbatis_mysql() -> Result<()> {
    let _ = GLOBAL_RBATIS_MYSQL
        .exec("select 1 from dual", vec![])
        .await?;
    Ok(())
}

async fn init_global_etcd_client() -> Result<()> {
    let etcd_client = init_etcd_client().await?;
    let mut kv_out = None;
    unsafe {
        let _ = GLOBAL_ETCD.set(etcd_client);
        let kv = GLOBAL_ETCD
            .get_mut()
            .unwrap()
            .get("key", None)
            .await
            .unwrap();
        println!("kv:{:?}", kv);
        kv_out = Some(kv);
    }

    println!("{:?}", kv_out);
    Ok(())
}
