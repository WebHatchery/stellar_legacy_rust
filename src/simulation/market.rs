//! Market trading and yearly price drift (GDD §5.1 "Keep as-is").

use crate::data::ResourceDelta;
use crate::state::sim::{base_price, SimState, TradeResource};

/// A price the exchange can show before the player commits. Keeping the
/// modifier factors beside the total lets the UI explain why the effective
/// price differs from the raw market ticker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TradeQuote {
    pub amount: i64,
    pub market_unit_price: f32,
    pub effective_unit_price: f32,
    pub total_credits: i64,
    pub reputation_factor: f32,
    /// Buy-side desperation premium or sell-side distress discount.
    pub pressure_factor: f32,
}

/// Yearly random walk on each tradeable's price, bounded to 0.5x-3x base.
pub fn drift_prices(sim: &mut SimState) {
    for i in 0..sim.market.entries.len() {
        let drift = sim.rng.range_f32(-0.08, 0.08);
        let entry = &mut sim.market.entries[i];
        let old = entry.price;
        let base = base_price(entry.resource);
        entry.price = (old * (1.0 + drift)).clamp(base * 0.5, base * 3.0);
        entry.trend = entry.price - old;
    }
}

pub fn price_of(sim: &SimState, resource: TradeResource) -> f32 {
    sim.market
        .entries
        .iter()
        .find(|e| e.resource == resource)
        .map(|e| e.price)
        .unwrap_or_else(|| base_price(resource))
}

fn trade_delta(resource: TradeResource, amount: i64, credits: i64) -> ResourceDelta {
    let mut delta = ResourceDelta {
        credits,
        ..Default::default()
    };
    match resource {
        TradeResource::Energy => delta.energy = amount,
        TradeResource::Minerals => delta.minerals = amount,
        TradeResource::Food => delta.food = amount,
        TradeResource::Influence => delta.influence = amount,
    }
    delta
}

/// The factor the ship's *name* puts on a trade price (content-depth provisioning round 30): the
/// market's regard for who it deals with, on the ship's `mercy` reputation — a well-regarded hull
/// is dealt with squarely, a feared one draws a risk premium. On a buy the price is scaled by
/// `1 - scale·(mercy - 0.5)` (a merciful ship pays less), on a sell by `1 + scale·(mercy - 0.5)`
/// (a merciful ship earns more), floored so no name ever makes a good free or worthless. 1.0
/// (inert) when the scale is 0 or the name is neutral. Reads reputation only; deterministic.
fn reputation_trade_factor(sim: &SimState, buying: bool) -> f32 {
    let scale = sim.market.trade_reputation_scale;
    if scale == 0.0 {
        return 1.0;
    }
    let standing = sim.reputation("mercy") - 0.5;
    let factor = if buying {
        1.0 - scale * standing
    } else {
        1.0 + scale * standing
    };
    factor.max(0.1)
}

/// The premium the market puts on a *buy* the ship makes while critically low on that good
/// (content-depth provisioning round 32): the market's regard for the ship's *need*. A ship
/// buying a survival good (food, energy) with its own stock below the resource's floor is over a
/// barrel, and the waystation prices the desperation in — the buy costs `1 + desperation_premium`.
/// Minerals and influence have no survival floor and never read as desperate; a sell is never
/// desperate (a low ship is not selling what it lacks). 1.0 (inert) when the premium is 0, the
/// floor unset, or the hold still comfortable. Reads the ship's own stock; deterministic.
fn desperation_factor(sim: &SimState, resource: TradeResource) -> f32 {
    let premium = sim.market.desperation_premium;
    if premium == 0.0 {
        return 1.0;
    }
    let (stock, floor) = match resource {
        TradeResource::Food => (sim.resources.food, sim.market.desperation_food_floor),
        TradeResource::Energy => (sim.resources.energy, sim.market.desperation_energy_floor),
        // No survival floor: the ship is never *desperate* for these, only short.
        TradeResource::Minerals | TradeResource::Influence => return 1.0,
    };
    if floor > 0 && stock < floor {
        1.0 + premium
    } else {
        1.0
    }
}

pub fn buy(sim: &mut SimState, resource: TradeResource, amount: i64) -> Result<(), String> {
    if amount <= 0 {
        return Err("Trade amount must be positive".to_owned());
    }
    let cost = buy_quote(sim, resource, amount).total_credits;
    let delta = trade_delta(resource, amount, -cost);
    if !sim.resources.can_afford(&delta) {
        return Err(format!("Need {cost} credits"));
    }
    sim.resources.apply(&delta);
    // The ship's own demand moves the thin local market (content-depth provisioning
    // round 22): buying up a good drives its price up against the next lot.
    shift_price(sim, resource, amount);
    Ok(())
}

/// The discount the market takes on a *sell* the ship makes while its treasury is critically bare
/// (content-depth provisioning round 33): the sell-side mirror of the it32 buy desperation. Where a
/// ship *buying* what it is low on is gouged, a ship *selling* its stores because it is broke —
/// credits below `distress_credit_floor` — is lowballed: the trader smells a fire sale and pays
/// `1 - distress_discount`. It reads the ship's *credits*, not the good on offer (a distress sale is
/// about the seller's need for cash, whatever they are selling), so it applies to every resource.
/// 1.0 (inert) when the discount is 0, the floor unset, or the treasury still comfortable.
fn distress_factor(sim: &SimState) -> f32 {
    let discount = sim.market.distress_discount;
    let floor = sim.market.distress_credit_floor;
    if discount == 0.0 || floor <= 0 {
        return 1.0;
    }
    if sim.resources.credits < floor {
        (1.0 - discount).max(0.0)
    } else {
        1.0
    }
}

/// Exact buy quote used by both the visible exchange and transaction service.
pub fn buy_quote(sim: &SimState, resource: TradeResource, amount: i64) -> TradeQuote {
    let market_unit_price = price_of(sim, resource);
    let reputation_factor = reputation_trade_factor(sim, true);
    let pressure_factor = desperation_factor(sim, resource);
    let effective_unit_price = market_unit_price * reputation_factor * pressure_factor;
    TradeQuote {
        amount,
        market_unit_price,
        effective_unit_price,
        total_credits: (effective_unit_price * amount as f32).ceil() as i64,
        reputation_factor,
        pressure_factor,
    }
}

/// Exact sell quote used by both the visible exchange and transaction service.
pub fn sell_quote(sim: &SimState, resource: TradeResource, amount: i64) -> TradeQuote {
    let market_unit_price = price_of(sim, resource);
    let reputation_factor = reputation_trade_factor(sim, false);
    let pressure_factor = distress_factor(sim);
    let effective_unit_price = market_unit_price * reputation_factor * pressure_factor;
    TradeQuote {
        amount,
        market_unit_price,
        effective_unit_price,
        total_credits: (effective_unit_price * amount as f32).floor() as i64,
        reputation_factor,
        pressure_factor,
    }
}

pub fn sell(sim: &mut SimState, resource: TradeResource, amount: i64) -> Result<(), String> {
    if amount <= 0 {
        return Err("Trade amount must be positive".to_owned());
    }
    let proceeds = sell_quote(sim, resource, amount).total_credits;
    let delta = trade_delta(resource, -amount, proceeds);
    if !sim.resources.can_afford(&delta) {
        return Err(format!("Not enough {} to sell", resource.label()));
    }
    sim.resources.apply(&delta);
    // …and dumping a surplus floods the market and drives its price down (round 22).
    shift_price(sim, resource, -amount);
    Ok(())
}

/// Move a resource's local price by the ship's own trade (content-depth provisioning
/// round 22): a positive `signed_amount` (a buy) pushes it up, a negative one (a sell)
/// down, scaled by the resource's base price and `market.impact_per_unit`, clamped to
/// the same 0.5x-3x band the yearly drift honours. The drift then walks it back toward
/// base over the following years, so a bulk trade's mark on the market is real but
/// temporary. Inert when `impact_per_unit` is 0.
fn shift_price(sim: &mut SimState, resource: TradeResource, signed_amount: i64) {
    let k = sim.market.impact_per_unit;
    if k == 0.0 {
        return;
    }
    let base = base_price(resource);
    let shift = base * k * signed_amount as f32;
    if let Some(entry) = sim
        .market
        .entries
        .iter_mut()
        .find(|e| e.resource == resource)
    {
        entry.price = (entry.price + shift).clamp(base * 0.5, base * 3.0);
    }
}

#[cfg(test)]
mod tests;
