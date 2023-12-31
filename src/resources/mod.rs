mod embed_resources;
mod etcd_resources;
mod init_resources;
mod mysql_rbatis_resource;
mod redis_resource;

pub use embed_resources::get_rbac_model;
pub use embed_resources::get_rbac_policy;
pub use init_resources::init_resources;

pub use etcd_resources::*;
pub use init_resources::*;
pub use mysql_rbatis_resource::init_datasource_mysql;
pub use redis_resource::InstanceType;
pub use redis_resource::RedisInstance;
