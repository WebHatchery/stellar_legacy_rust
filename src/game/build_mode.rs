//! Small boundary for behavior that differs between the full release and the
//! feature-gated storefront demo.

use crate::data::GameData;

pub(super) fn load_data() -> GameData {
    let mut data = GameData::load()
        .unwrap_or_else(|err| panic!("Stellar Legacy embedded data failed to load: {err}"));
    // Browser storefront origins usually isolate persistence, but a locally
    // run demo must not share slots or preferences with the full game.
    if crate::data::contracts::is_demo_build() {
        data.config.game_name.push_str("_demo");
    }
    data
}
