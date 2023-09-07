use logger::init_log;
mod cmd;
mod commons;
mod configure;
mod errors;
mod grpcserver;
mod httpserver;
mod interact;
mod logger;
mod privilege;
mod resources;
mod service;

fn main() {
    init_log();
    cmd::run_app();
}
