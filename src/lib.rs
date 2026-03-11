pub mod app;
pub mod cli;
pub mod domain;
pub mod io;
pub mod storage;
#[cfg(test)]
pub(crate) mod test_support;

pub fn run() -> i32 {
    app::service::run()
}
