//! The new-game screen (W7): the founding charter. The player picks a legacy
//! (the founding society's mandate) and the founding peoples who crew the
//! ship. Each choice shows what it means in play — a legacy's temperament and
//! undoing, a people's character, stance, craft, kin and rivals — so the
//! picks read as consequential rather than as a list of names.

use super::super::*;
use super::MenuCtx;
use macroquad_toolkit::ui::{draw_text_block_ex, measure_text_size, wrap_text};

const PAD: f32 = 14.0;
const NAME_SIZE: f32 = 18.0;
const BODY_SIZE: f32 = 16.0;
const META_SIZE: f32 = 15.0;
/// Fitted text never shrinks below this, so a crowded block truncates rather
/// than turning to dust.
const FIT_MIN: f32 = 13.0;

/// Measured line height for a size at the current text-scale preference.
fn line_h(size: f32) -> f32 {
    measure_text_size("Ag", TextStyle::new(size, term::primary())).height
}

/// Height of `text` wrapped to `w` as `draw_text_block` draws it at full size.
fn wrapped_h(text: &str, w: f32, size: f32, gap: f32) -> f32 {
    let style = TextStyle::new(size, term::primary()).with_line_gap(gap);
    let step = style.effective_font_size() + style.effective_line_gap();
    wrap_text(text, w, size).len() as f32 * step + 4.0
}

/// One line that shrinks to fit its width instead of wrapping or running past
/// it; at the floor size it is cut with an ellipsis. Returns the height used.
fn fit_line(text: &str, x: f32, top: f32, w: f32, size: f32, color: Color) -> f32 {
    let natural = measure_text_size(text, TextStyle::new(size, color)).width;
    let size = if natural > w && natural > 0.0 {
        (size * w / natural).max(FIT_MIN)
    } else {
        size
    };
    let style = TextStyle::new(size, color);
    let mut shown = text.to_owned();
    while measure_text_size(&shown, style).width > w && shown.chars().count() > 1 {
        shown.pop();
        while shown.ends_with(' ') {
            shown.pop();
        }
        shown.pop();
        shown.push('…');
    }
    let ascent = measure_text_size("Ag", style).offset_y;
    draw_ui_text_ex(&shown, x, top + 3.0 + ascent, style.params());
    line_h(size) + 6.0
}

fn fit_block(text: &str, rect: Rect, size: f32, color: Color) {
    if rect.h <= 0.0 {
        return;
    }
    draw_text_block_ex(
        text,
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        TextStyle::new(size, color).with_line_gap(3.0),
        FIT_MIN,
    );
}

fn card(rect: Rect, selected: bool) {
    draw_surface(
        rect,
        &SurfaceStyle::new(if selected {
            term::surface_active()
        } else {
            term::surface_inset()
        })
        .with_border(
            1.0,
            if selected {
                term::primary()
            } else {
                term::faint()
            },
        ),
    );
}

pub(super) fn draw_new_game(ctx: &MenuCtx<'_>) -> Vec<UiAction> {
    let mut actions = Vec::new();

    // No wordmark here: the main menu carries the title art, and the vertical
    // room goes to the peoples' cards instead.
    let panel = Rect::new(100.0, 24.0, logical_width() - 200.0, 1032.0);
    term_panel(panel, Some("FOUNDING CHARTER"));
    let content = panel.inset(24.0);
    let mut top = content.y + 22.0;
    top += draw_inheritance(ctx, content.x, top, content.w);

    let buttons_h = 48.0;
    let columns = Rect::new(
        content.x,
        top,
        content.w,
        content.bottom() - buttons_h - 16.0 - top,
    );
    let left_w = 500.0;
    let gap = 28.0;
    let left = Rect::new(columns.x, columns.y, left_w, columns.h);
    let right = Rect::new(
        columns.x + left_w + gap,
        columns.y,
        columns.w - left_w - gap,
        columns.h,
    );
    draw_legacy_column(ctx, left, &mut actions);
    draw_peoples_column(ctx, right, &mut actions);
    draw_buttons(ctx, content, buttons_h, &mut actions);
    actions
}

/// What this dynasty inherits from the Chronicle of past voyages, in words.
/// Hidden for a first dynasty: an empty inheritance is not worth a line.
fn draw_inheritance(ctx: &MenuCtx<'_>, x: f32, top: f32, w: f32) -> f32 {
    let heritage = crate::heritage::derive(ctx.chronicle, &ctx.data.config.heritage);
    if !heritage.has_bonus() {
        return 0.0;
    }
    let voyages = ctx.chronicle.entries.len();
    let mut gifts = Vec::new();
    if heritage.credits != 0 {
        gifts.push(format!("{:+} credits", heritage.credits));
    }
    if heritage.influence != 0 {
        gifts.push(format!("{:+} influence", heritage.influence));
    }
    if heritage.tradition != 0 {
        gifts.push(format!("{:+} tradition", heritage.tradition));
    }
    let line = format!(
        "{} voyage{} remembered · departs with {}",
        voyages,
        if voyages == 1 { "" } else { "s" },
        gifts.join(", ")
    );
    let label = format!("INHERITANCE · {} LINE", heritage.tier_name.to_uppercase());
    let label_style = TextStyle::new(META_SIZE, term::accent());
    let h = line_h(BODY_SIZE) + 20.0;
    let strip = Rect::new(x, top, w, h);
    draw_surface(
        strip,
        &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
    );
    let label_w = measure_text_size(&label, label_style).width;
    let baseline = strip.y + strip.h / 2.0 + line_h(META_SIZE) * 0.35;
    draw_ui_text_ex(&label, strip.x + PAD, baseline, label_style.params());
    let text_x = strip.x + PAD + label_w + 24.0;
    fit_line(
        &line,
        text_x,
        strip.y + 6.0,
        strip.right() - PAD - text_x,
        BODY_SIZE,
        term::primary(),
    );
    h + 16.0
}

/// Column heading and its one-paragraph explanation; returns the height used.
fn draw_heading(heading: &str, heading_color: Color, intro: &str, col: Rect) -> f32 {
    let head_h = line_h(17.0);
    draw_ui_text_ex(
        heading,
        col.x,
        col.y + head_h,
        TextStyle::new(17.0, heading_color).params(),
    );
    let intro_h = wrapped_h(intro, col.w, META_SIZE, 3.0);
    fit_block(
        intro,
        Rect::new(col.x, col.y + head_h + 8.0, col.w, intro_h),
        META_SIZE,
        term::dim(),
    );
    head_h + 8.0 + intro_h + 10.0
}

/// Legacy cards: each closed to its name until chosen. Only the chosen legacy
/// opens to its description, and only it shows what it changes.
fn draw_legacy_column(ctx: &MenuCtx<'_>, col: Rect, actions: &mut Vec<UiAction>) {
    let tut = &ctx.data.config.tutorial;
    let mut y = col.y + draw_heading("LEGACY", term::accent(), &tut.legacy_intro, col);
    let name_h = line_h(NAME_SIZE);
    let body_w = col.w - PAD * 2.0;
    for (i, legacy_id) in ctx.legacy_ids.iter().enumerate() {
        let Some(legacy) = ctx.data.legacies.get(legacy_id) else {
            continue;
        };
        let selected = i == ctx.menu.selected_legacy;
        let body_h = if selected {
            wrapped_h(&legacy.description, body_w, BODY_SIZE, 3.0) + 8.0
        } else {
            0.0
        };
        let rect = Rect::new(col.x, y, col.w, PAD * 2.0 + name_h + body_h);
        card(rect, selected);
        draw_ui_text_ex(
            &format!("{} {}", i + 1, legacy.name),
            rect.x + PAD,
            rect.y + PAD + name_h,
            TextStyle::new(
                NAME_SIZE,
                if selected {
                    term::accent()
                } else {
                    term::primary()
                },
            )
            .params(),
        );
        if selected {
            fit_block(
                &legacy.description,
                Rect::new(rect.x + PAD, rect.y + PAD + name_h + 8.0, body_w, body_h),
                BODY_SIZE,
                term::primary(),
            );
        }
        if ctx.pointer.released_on(rect) {
            actions.push(UiAction::SelectLegacy(i));
        }
        y += rect.h + 10.0;
    }
    if let Some(legacy) = ctx
        .legacy_ids
        .get(ctx.menu.selected_legacy)
        .and_then(|id| ctx.data.legacies.get(id))
    {
        draw_legacy_effects(legacy, Rect::new(col.x, y, col.w, col.bottom() - y));
    }
}

/// The chosen legacy's temperament, then its undoing in the alert tone. The
/// authored `effects` line carries both, split at "Undoing:". The box is only
/// as tall as its text; with large text the undoing drops before the
/// temperament is cut.
fn draw_legacy_effects(legacy: &crate::data::legacies::LegacyDef, room: Rect) {
    let (temperament, undoing) = match legacy.effects.split_once("Undoing:") {
        Some((t, u)) => (t.trim(), Some(u.trim())),
        None => (legacy.effects.trim(), None),
    };
    let w = room.w - PAD * 2.0;
    let head_h = line_h(META_SIZE);
    let mut sections = vec![(
        "WHAT THIS CHANGES".to_owned(),
        term::accent(),
        temperament.to_owned(),
        term::primary(),
    )];
    if let Some(undoing) = undoing {
        let (name, text) = undoing
            .split_once(" — ")
            .map_or(("", undoing), |(n, t)| (n.trim(), t.trim()));
        let label = if name.is_empty() {
            "UNDOING".to_owned()
        } else {
            format!("UNDOING · {name}")
        };
        let mut chars = text.chars();
        let text = chars
            .next()
            .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
            .unwrap_or_default();
        sections.push((label, term::alert(), text, term::dim()));
    }
    let section_h = |body: &str| head_h + 8.0 + wrapped_h(body, w, BODY_SIZE, 3.0);
    let mut total = PAD * 2.0 - 12.0;
    let mut shown = 0;
    for (_, _, body, _) in &sections {
        let h = section_h(body) + 12.0;
        if shown > 0 && total + h > room.h {
            break;
        }
        total += h;
        shown += 1;
    }
    let rect = Rect::new(room.x, room.y, room.w, total.min(room.h));
    if rect.h < 40.0 {
        return;
    }
    draw_surface(
        rect,
        &SurfaceStyle::new(term::surface_inset()).with_border(1.0, term::faint()),
    );
    let mut y = rect.y + PAD;
    for (label, tone, body, body_tone) in sections.iter().take(shown) {
        draw_ui_text_ex(
            label,
            rect.x + PAD,
            y + head_h,
            TextStyle::new(META_SIZE, *tone).params(),
        );
        let body_top = y + head_h + 8.0;
        let body_h = (rect.bottom() - PAD - body_top).min(wrapped_h(body, w, BODY_SIZE, 3.0));
        fit_block(
            body,
            Rect::new(rect.x + PAD, body_top, w, body_h),
            BODY_SIZE,
            *body_tone,
        );
        y += section_h(body) + 12.0;
    }
}

/// The peoples as a 2-column grid. Unchosen peoples are a name and a stance
/// tag; a chosen people opens to its full card. Rows are as tall as their
/// tallest card, and open rows share whatever room is left when text is large.
fn draw_peoples_column(ctx: &MenuCtx<'_>, col: Rect, actions: &mut Vec<UiAction>) {
    let starting = ctx.data.config.factions.starting_count as usize;
    let chosen = ctx.menu.selected_factions.len();
    let heading = format!("FOUNDING PEOPLES · CHOOSE {starting}  ({chosen}/{starting})");
    let tone = if chosen == starting {
        term::accent()
    } else {
        term::primary()
    };
    let tut = &ctx.data.config.tutorial;
    let used = draw_heading(&heading, tone, &tut.factions_intro, col);

    let peoples: Vec<(String, &crate::data::factions::FactionDef)> =
        GameData::sorted_ids(&ctx.data.factions)
            .into_iter()
            .filter_map(|id| ctx.data.factions.get(&id).map(|f| (id, f)))
            .collect();
    let cols = 2usize;
    let gap = 12.0;
    let grid_top = col.y + used;
    let card_w = (col.w - gap * (cols as f32 - 1.0)) / cols as f32;
    let is_chosen = |id: &str| ctx.menu.selected_factions.iter().any(|f| f == id);
    let need: Vec<f32> = peoples
        .iter()
        .map(|(id, faction)| {
            if is_chosen(id) {
                open_card_height(ctx, id, faction, card_w)
            } else {
                closed_card_height()
            }
        })
        .collect();

    // Closed rows keep their size; open rows split what remains.
    let rows: Vec<(f32, bool)> = need
        .chunks(cols)
        .zip(peoples.chunks(cols))
        .map(|(h, row)| {
            (
                h.iter().copied().fold(0.0, f32::max),
                row.iter().any(|(id, _)| is_chosen(id)),
            )
        })
        .collect();
    let avail = col.bottom() - grid_top - gap * rows.len().saturating_sub(1) as f32;
    let closed_sum: f32 = rows.iter().filter(|r| !r.1).map(|r| r.0).sum();
    let open_count = rows.iter().filter(|r| r.1).count().max(1) as f32;
    let open_share = (avail - closed_sum) / open_count;

    let mut y = grid_top;
    for (r, (row_h, open)) in rows.iter().enumerate() {
        let row_h = if *open { row_h.min(open_share) } else { *row_h };
        for c in 0..cols {
            let i = r * cols + c;
            let Some((id, faction)) = peoples.get(i) else {
                continue;
            };
            let rect = Rect::new(
                col.x + c as f32 * (card_w + gap),
                y,
                card_w,
                need[i].min(row_h),
            );
            draw_people_card(ctx, id, faction, rect);
            if ctx.pointer.released_on(rect) {
                actions.push(UiAction::ToggleFaction(id.clone()));
            }
        }
        y += row_h + gap;
    }
}

fn meta_step() -> f32 {
    line_h(META_SIZE) + 6.0
}

fn closed_card_height() -> f32 {
    PAD * 2.0 + line_h(NAME_SIZE) + 4.0 + meta_step()
}

fn open_card_height(
    ctx: &MenuCtx<'_>,
    id: &str,
    faction: &crate::data::factions::FactionDef,
    w: f32,
) -> f32 {
    let desc_h = wrapped_h(&faction.description, w - PAD * 2.0, BODY_SIZE, 3.0);
    let meta = people_meta(ctx, id, faction).len() as f32;
    PAD * 2.0 + line_h(NAME_SIZE) + 10.0 + desc_h + 6.0 + meta_step() * meta - 2.0
}

/// What a people means in play, most important first: stance and the module
/// they tend, their kin and rivals (alert tone when a chosen people's rival is
/// chosen too), how the ship's name drifts when they lead, and their growth.
fn people_meta(
    ctx: &MenuCtx<'_>,
    id: &str,
    faction: &crate::data::factions::FactionDef,
) -> Vec<(String, Color)> {
    let chosen = &ctx.menu.selected_factions;
    let selected = chosen.iter().any(|f| f == id);
    let name_of = |other: &String| {
        ctx.data
            .factions
            .get(other)
            .map(|f| f.name.trim_start_matches("The ").to_owned())
    };
    let mut meta: Vec<(String, Color)> = Vec::new();
    let mut role = stance_label(faction.ideology).to_owned();
    if let Some(module) = ctx.data.subsystems.get(&faction.tended_subsystem) {
        role.push_str(&format!(" · tends {}", module.name));
    }
    meta.push((role, term::dim()));
    let mut ties = Vec::new();
    let kin: Vec<String> = faction.allies.iter().filter_map(name_of).collect();
    if !kin.is_empty() {
        ties.push(format!("Kin: {}", kin.join(", ")));
    }
    let rivals: Vec<String> = faction.rivals.iter().filter_map(name_of).collect();
    if !rivals.is_empty() {
        ties.push(format!("At odds: {}", rivals.join(", ")));
    }
    if !ties.is_empty() {
        let clash = selected && faction.rivals.iter().any(|r| chosen.contains(r));
        meta.push((
            ties.join(" · "),
            if clash { term::alert() } else { term::dim() },
        ));
    }
    if let Some(helm) = helm_line(faction) {
        meta.push((helm, term::dim()));
    }
    if faction.growth_bias > 0.0 {
        meta.push((
            "Grows in number over the generations".to_owned(),
            term::dim(),
        ));
    } else if faction.growth_bias < 0.0 {
        meta.push((
            "Dwindles in number over the generations".to_owned(),
            term::dim(),
        ));
    }
    meta
}

/// One founding people. Closed: lamp, name and stance tag. Open (chosen): the
/// full description, then its meta lines anchored to the card's foot; when
/// larger text leaves too little room the least important lines drop first.
fn draw_people_card(
    ctx: &MenuCtx<'_>,
    id: &str,
    faction: &crate::data::factions::FactionDef,
    rect: Rect,
) {
    let selected = ctx.menu.selected_factions.iter().any(|f| f == id);
    card(rect, selected);

    let name_h = line_h(NAME_SIZE);
    let lamp_r = 6.0;
    let lamp_c = vec2(rect.x + PAD + lamp_r, rect.y + PAD + name_h * 0.55);
    if selected {
        draw_circle(lamp_c.x, lamp_c.y, lamp_r, term::accent());
    } else {
        draw_circle_lines(lamp_c.x, lamp_c.y, lamp_r, 1.5, term::dim());
    }
    let name_x = rect.x + PAD + lamp_r * 2.0 + 10.0;
    fit_line(
        &faction.name,
        name_x,
        rect.y + PAD - 4.0,
        rect.right() - PAD - name_x,
        NAME_SIZE,
        if selected {
            term::accent()
        } else {
            term::primary()
        },
    );
    let inner_w = rect.w - PAD * 2.0;
    if !selected {
        fit_line(
            stance_label(faction.ideology),
            name_x,
            rect.y + PAD + name_h + 2.0,
            rect.right() - PAD - name_x,
            META_SIZE,
            term::dim(),
        );
        return;
    }

    let mut meta = people_meta(ctx, id, faction);
    let body_top = rect.y + PAD + name_h + 10.0;
    let desc_h = wrapped_h(&faction.description, inner_w, BODY_SIZE, 3.0);
    let step = meta_step();
    let room = rect.bottom() - PAD + 2.0 - (body_top + desc_h + 6.0);
    let fits = ((room / step).floor().max(1.0) as usize).min(meta.len());
    meta.truncate(fits);
    let meta_top = rect.bottom() - PAD + 2.0 - step * meta.len() as f32;
    for (n, (text, color)) in meta.iter().enumerate() {
        fit_line(
            text,
            rect.x + PAD,
            meta_top + step * n as f32,
            inner_w,
            META_SIZE,
            *color,
        );
    }
    fit_block(
        &faction.description,
        Rect::new(rect.x + PAD, body_top, inner_w, meta_top - 6.0 - body_top),
        BODY_SIZE,
        term::primary(),
    );
}

fn draw_buttons(ctx: &MenuCtx<'_>, content: Rect, h: f32, actions: &mut Vec<UiAction>) {
    let starting = ctx.data.config.factions.starting_count as usize;
    let chosen = ctx.menu.selected_factions.len();
    let by = content.bottom() - h;
    let w = 320.0;
    // Destructive action on the far left; the way forward on the far right.
    if term_button(
        Rect::new(content.x, by, w, h),
        "DELETE SAVE",
        ctx.menu.save_exists,
        ctx.pointer,
    ) {
        actions.push(UiAction::DeleteSave);
    }
    if term_button(
        Rect::new(content.right() - w * 2.0 - 16.0, by, w, h),
        "BACK [ESC]",
        true,
        ctx.pointer,
    ) {
        actions.push(UiAction::BackToMainMenu);
    }
    if term_button(
        Rect::new(content.right() - w, by, w, h),
        "BEGIN VOYAGE [ENTER]",
        chosen == starting,
        ctx.pointer,
    ) {
        actions.push(UiAction::StartNewGame);
    }
}

/// How the ship's name drifts while this people is the largest aboard, from
/// their authored reputation leanings. `None` for a people whose rule leaves
/// the ship's name to its choices alone.
fn helm_line(faction: &crate::data::factions::FactionDef) -> Option<String> {
    let mut leanings: Vec<_> = faction
        .reputation_leanings
        .iter()
        .filter(|(_, v)| **v != 0.0)
        .collect();
    if leanings.is_empty() {
        return None;
    }
    leanings.sort_by(|a, b| a.0.cmp(b.0));
    let drifts: Vec<String> = leanings
        .into_iter()
        .map(|(trait_id, v)| match (trait_id.as_str(), *v > 0.0) {
            ("mercy", true) => "the ship grows a kinder name".to_owned(),
            ("mercy", false) => "the ship grows a harder name".to_owned(),
            (other, true) => format!("the ship's {other} rises"),
            (other, false) => format!("the ship's {other} falls"),
        })
        .collect();
    Some(format!("When they lead: {}", drifts.join(", ")))
}

/// A people's stance on technology, from their authored ideology.
fn stance_label(ideology: f32) -> &'static str {
    if ideology > 0.66 {
        "Radically tech-embracing"
    } else if ideology > 0.2 {
        "Tech-embracing"
    } else if ideology >= -0.2 {
        "Pragmatic middle"
    } else if ideology >= -0.66 {
        "Tech-averse"
    } else {
        "Tech-averse, traditional"
    }
}
