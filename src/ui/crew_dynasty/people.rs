//! People get separate reading spaces for family, officers, society and command.
use super::*;
use crate::ui::identity;

pub fn draw(ctx: &GameplayCtx<'_>, area: Rect, pointer: Pointer, actions: &mut Vec<UiAction>) {
    match ctx.presentation.people_page.get() {
        1 => draw_posts(ctx, area, pointer, actions),
        2 => draw_factions(ctx, area, pointer, actions),
        3 => draw_council(ctx, area, pointer, actions),
        _ => {
            let captain = ctx.sim.dynasty.leader();
            let heir = planned_heir(&ctx.sim.dynasty, &ctx.data.config);
            let half = (area.w - 100.0) * 0.5;
            for (index, (title, person)) in
                [("Serving captain", captain), ("Next eligible heir", heir)]
                    .into_iter()
                    .enumerate()
            {
                let card = Rect::new(area.x + index as f32 * (half + 100.0), area.y, half, 152.0);
                term_panel(card, Some(title));
                if let Some(person) = person {
                    identity::portrait(
                        Rect::new(card.x + 18.0, card.y + 54.0, 70.0, 78.0),
                        &person.name,
                    );
                    draw_text_block(
                        &person.name,
                        card.x + 108.0,
                        card.y + 48.0,
                        card.w - 126.0,
                        50.0,
                        24.0,
                        4.0,
                        term::primary(),
                    );
                    draw_ui_text_ex(
                        &format!("Age {} · Leadership {}", person.age, person.leadership),
                        card.x + 108.0,
                        card.y + 119.0,
                        TextStyle::new(16.0, term::dim()).params(),
                    );
                } else {
                    draw_text_block(
                        "No eligible successor. Review the family below.",
                        card.x + 20.0,
                        card.y + 58.0,
                        card.w - 40.0,
                        60.0,
                        18.0,
                        5.0,
                        term::alert(),
                    );
                }
            }
            let start = area.x + half + 14.0;
            let end = start + 70.0;
            draw_line(
                start,
                area.y + 85.0,
                end,
                area.y + 85.0,
                2.0,
                term::primary(),
            );
            draw_triangle(
                vec2(end, area.y + 85.0),
                vec2(end - 12.0, area.y + 79.0),
                vec2(end - 12.0, area.y + 91.0),
                term::primary(),
            );
            draw_roster(
                ctx,
                Rect::new(area.x, area.y + 172.0, area.w, area.h - 172.0),
                pointer,
                actions,
            );
        }
    }
}
