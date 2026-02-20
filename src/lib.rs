pub mod app;
pub mod cli;
pub mod domain;
pub mod io;
pub mod storage;

pub fn run() -> i32 {
    app::service::run()
}
