use ss::Gate;

#[cfg_attr(feature  = "hongkong", path = "hongkong.rs")]
mod default_config;

mod config;
mod log;
mod ss;
fn main() {
    log::init();
    Gate::new().start();
}
