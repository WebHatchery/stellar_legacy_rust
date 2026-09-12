//! Stellar Legacy's minimal Macroquad executable shell.

use stellar_legacy::{run, window_conf};

#[macroquad::main(window_conf)]
async fn main() {
    run().await;
}
