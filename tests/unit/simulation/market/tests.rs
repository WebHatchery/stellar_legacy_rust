use super::*;
use crate::data::GameData;
use crate::state::sim::SimState;

#[test]
fn buy_and_sell_move_credits_and_goods() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "wanderers",
        11,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let credits_before = sim.resources.credits;
    let food_before = sim.resources.food;

    let receipt = buy(&mut sim, TradeResource::Food, 100).unwrap();
    assert_eq!(sim.resources.food, food_before + 100);
    assert!(sim.resources.credits < credits_before);
    assert!(receipt.market_price_after > receipt.market_price_before);
    assert_eq!(sim.market.last_trade, Some(receipt));

    let receipt = sell(&mut sim, TradeResource::Food, 100).unwrap();
    assert_eq!(sim.resources.food, food_before);
    assert!(receipt.market_price_after < receipt.market_price_before);
    assert_eq!(sim.market.last_trade, Some(receipt));
}

#[test]
fn old_market_saves_default_to_no_settlement_ticket() {
    let data = GameData::load().unwrap();
    let sim = SimState::new_campaign(
        &data,
        "wanderers",
        12,
        &crate::state::sim::founding_faction_ids(&data),
    );
    let mut value = serde_json::to_value(sim).unwrap();
    value["market"]
        .as_object_mut()
        .unwrap()
        .remove("last_trade");

    let migrated: SimState = serde_json::from_value(value).unwrap();
    assert!(migrated.market.last_trade.is_none());
}

#[test]
fn displayed_quotes_are_the_exact_transaction_totals() {
    let data = GameData::load().unwrap();
    let mut buyer = SimState::new_campaign(
        &data,
        "wanderers",
        21,
        &crate::state::sim::founding_faction_ids(&data),
    );
    buyer.resources.food = 0;
    buyer.resources.credits = 1_000_000;
    buyer.reputation.insert("mercy".to_owned(), 0.8);
    let quote = buy_quote(&buyer, TradeResource::Food, 137);
    let credits_before = buyer.resources.credits;
    buy(&mut buyer, TradeResource::Food, quote.amount).unwrap();
    assert_eq!(
        credits_before - buyer.resources.credits,
        quote.total_credits
    );
    assert!(
        quote.pressure_factor > 1.0,
        "the visible quote includes need"
    );
    assert_ne!(
        quote.reputation_factor, 1.0,
        "the visible quote includes the ship's name"
    );

    let mut seller = SimState::new_campaign(
        &data,
        "wanderers",
        22,
        &crate::state::sim::founding_faction_ids(&data),
    );
    seller.resources.credits = 0;
    seller.resources.minerals = 1_000;
    seller.reputation.insert("mercy".to_owned(), 0.2);
    let quote = sell_quote(&seller, TradeResource::Minerals, 137);
    let credits_before = seller.resources.credits;
    sell(&mut seller, TradeResource::Minerals, quote.amount).unwrap();
    assert_eq!(
        seller.resources.credits - credits_before,
        quote.total_credits
    );
    assert!(
        quote.pressure_factor < 1.0,
        "the visible quote includes distress"
    );
}

#[test]
fn zero_or_negative_trades_are_rejected() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "wanderers",
        23,
        &crate::state::sim::founding_faction_ids(&data),
    );
    assert!(buy(&mut sim, TradeResource::Food, 0).is_err());
    assert!(sell(&mut sim, TradeResource::Food, -10).is_err());
}

#[test]
fn a_desperate_buy_of_a_survival_good_pays_a_premium() {
    // Content-depth provisioning round 32: the market reads the ship's need. Buying food with
    // the larder near famine costs a premium a comfortable buyer never pays — so buy early,
    // before you are over a barrel. Minerals have no survival floor and never read as desperate.
    let data = GameData::load().unwrap();
    assert!(
        data.config.market_desperation_premium > 0.0,
        "this test needs the desperation premium enabled"
    );
    let floor = data.config.low_food_threshold;

    // The credits a 100-unit food buy costs at a given starting food stock (fresh sim each
    // time, so the round-22 price shift from one buy never bleeds into the next).
    let food_cost_at = |food: i64| -> i64 {
        let mut sim = SimState::new_campaign(
            &data,
            "wanderers",
            11,
            &crate::state::sim::founding_faction_ids(&data),
        );
        sim.resources.food = food;
        sim.resources.credits = 1_000_000;
        let before = sim.resources.credits;
        buy(&mut sim, TradeResource::Food, 100).unwrap();
        before - sim.resources.credits
    };

    let comfortable = food_cost_at(floor + 5_000); // a full larder: no desperation
    let desperate = food_cost_at(0); // near famine: the premium bites
    assert!(
        desperate > comfortable,
        "a starving ship pays a premium for food ({desperate} vs {comfortable})"
    );

    // Minerals carry no survival floor, so an empty mineral hold is short, not desperate.
    let mineral_cost_at = |minerals: i64| -> i64 {
        let mut sim = SimState::new_campaign(
            &data,
            "wanderers",
            11,
            &crate::state::sim::founding_faction_ids(&data),
        );
        sim.resources.minerals = minerals;
        sim.resources.credits = 1_000_000;
        let before = sim.resources.credits;
        buy(&mut sim, TradeResource::Minerals, 100).unwrap();
        before - sim.resources.credits
    };
    assert_eq!(
        mineral_cost_at(0),
        mineral_cost_at(100_000),
        "a mineral buy reads no desperation, however empty the hold"
    );
}

#[test]
fn a_broke_ship_sells_at_a_distress_discount() {
    // Content-depth provisioning round 33: the sell-side mirror of the buy desperation. A ship
    // selling its stores because the coffers are bare earns less than a solvent one selling the
    // same lot — the trader smells a fire sale. The discount reads the ship's credits, not the
    // good, so it would bite whatever the ship sells.
    let data = GameData::load().unwrap();
    assert!(
        data.config.market_distress_discount > 0.0,
        "this test needs the distress discount enabled"
    );
    let floor = data.config.distress_credit_floor;

    // The proceeds of a 100-unit food sell at a given starting credit level (fresh sim each
    // time, so the round-22 price shift never bleeds between sales).
    let proceeds_at = |credits: i64| -> i64 {
        let mut sim = SimState::new_campaign(
            &data,
            "wanderers",
            11,
            &crate::state::sim::founding_faction_ids(&data),
        );
        sim.resources.food = 10_000;
        sim.resources.credits = credits;
        let before = sim.resources.credits;
        sell(&mut sim, TradeResource::Food, 100).unwrap();
        sim.resources.credits - before
    };

    let solvent = proceeds_at(floor + 5_000); // comfortable coffers: full price
    let broke = proceeds_at(0); // bare coffers: the fire-sale discount bites
    assert!(
        broke < solvent,
        "a broke ship's fire sale earns less than a solvent ship's ({broke} vs {solvent})"
    );
    assert!(
        broke > 0,
        "but the stores are not taken for nothing ({broke})"
    );
}

#[test]
fn cannot_sell_more_than_held() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "wanderers",
        11,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.resources.influence = 5;
    assert!(sell(&mut sim, TradeResource::Influence, 50).is_err());
}

#[test]
fn the_ships_own_trades_move_the_thin_local_market() {
    // Content-depth provisioning round 22: a lone ship is a whale in a waypoint
    // market — stocking up drives a price up, dumping a surplus drives it down, both
    // clamped to the drift's band.
    let data = GameData::load().unwrap();
    assert!(
        data.config.market_impact_per_unit > 0.0,
        "this test needs the market-impact coupling enabled"
    );
    let mut sim = SimState::new_campaign(
        &data,
        "wanderers",
        3,
        &crate::state::sim::founding_faction_ids(&data),
    );
    sim.resources.credits = 1_000_000;
    sim.resources.minerals = 100_000;

    // Buying up minerals drives their price up against the next lot.
    let before = price_of(&sim, TradeResource::Minerals);
    buy(&mut sim, TradeResource::Minerals, 1_000).unwrap();
    let after_buy = price_of(&sim, TradeResource::Minerals);
    assert!(
        after_buy > before,
        "buying a bulk lot drives the price up: {before} -> {after_buy}"
    );
    let entry = sim
        .market
        .entries
        .iter()
        .find(|entry| entry.resource == TradeResource::Minerals)
        .unwrap();
    assert_eq!(entry.trend, after_buy - before);

    // Dumping a surplus floods the market and drives the price back down.
    sell(&mut sim, TradeResource::Minerals, 3_000).unwrap();
    let after_sell = price_of(&sim, TradeResource::Minerals);
    assert!(
        after_sell < after_buy,
        "dumping a surplus drives the price down: {after_buy} -> {after_sell}"
    );

    // The impact never breaks the drift's 0.5x-3x bounds.
    let base = base_price(TradeResource::Minerals);
    buy(&mut sim, TradeResource::Minerals, 100_000).unwrap();
    assert!(
        price_of(&sim, TradeResource::Minerals) <= base * 3.0,
        "even a whale trade stays inside the price band"
    );
}

#[test]
fn a_ships_name_bends_its_trade_terms() {
    // Content-depth provisioning round 30: the market prices for who it deals with. A
    // merciful, well-regarded hull buys cheaper and sells dearer; a feared one draws a risk
    // premium (buys dear, sells cheap); a neutral name trades at the base. Isolated by
    // comparing fresh ships that differ only in reputation, on the same small trade.
    let data = GameData::load().unwrap();
    assert!(
        data.config.trade_reputation_scale > 0.0,
        "this test needs the reputation-trade coupling enabled"
    );
    let fresh = || {
        SimState::new_campaign(
            &data,
            "wanderers",
            5,
            &crate::state::sim::founding_faction_ids(&data),
        )
    };
    let buy_cost = |mercy: f32| -> i64 {
        let mut sim = fresh();
        sim.resources.credits = 1_000_000;
        sim.reputation.insert("mercy".to_string(), mercy);
        let before = sim.resources.credits;
        buy(&mut sim, TradeResource::Minerals, 100).unwrap();
        before - sim.resources.credits
    };
    let (merciful, neutral, feared) = (buy_cost(1.0), buy_cost(0.5), buy_cost(0.0));
    assert!(
        merciful < neutral,
        "a well-regarded ship buys cheaper ({merciful} vs {neutral})"
    );
    assert!(
        feared > neutral,
        "a feared ship pays a risk premium ({feared} vs {neutral})"
    );

    let sell_proceeds = |mercy: f32| -> i64 {
        let mut sim = fresh();
        sim.resources.minerals = 100_000;
        sim.reputation.insert("mercy".to_string(), mercy);
        let before = sim.resources.credits;
        sell(&mut sim, TradeResource::Minerals, 100).unwrap();
        sim.resources.credits - before
    };
    assert!(
        sell_proceeds(1.0) > sell_proceeds(0.0),
        "a well-regarded ship sells dearer than a feared one"
    );
}

#[test]
fn prices_stay_within_bounds() {
    let data = GameData::load().unwrap();
    let mut sim = SimState::new_campaign(
        &data,
        "preservers",
        2,
        &crate::state::sim::founding_faction_ids(&data),
    );
    for _ in 0..200 {
        drift_prices(&mut sim);
    }
    for entry in &sim.market.entries {
        let base = base_price(entry.resource);
        assert!(entry.price >= base * 0.5 && entry.price <= base * 3.0);
    }
}
