//! Phone shell geometry keeps time, reading and navigation in separate regions.
use macroquad::prelude::Rect;

pub(super) struct Layout {
    pub title: Rect,
    pub status: Rect,
    pub pause: Rect,
    pub utilities: Rect,
    pub speeds: [Rect; 3],
    pub content: Rect,
    pub navigation: Rect,
    pub compact_navigation: bool,
    columns: usize,
    row_height: f32,
}

impl Layout {
    pub fn new(width: f32, height: f32, text_scale: f32, widest_label: f32) -> Self {
        let title = Rect::new(12.0, 4.0, width - 24.0, 30.0 * text_scale);
        let status = Rect::new(12.0, title.y + title.h, width - 24.0, 22.0 * text_scale);
        let control_y = status.y + status.h + 8.0;
        let control_h = (26.0 * text_scale + 18.0).max(44.0);
        let pause = Rect::new(12.0, control_y, (width - 32.0) * 0.5, control_h);
        let utilities = Rect::new(pause.x + pause.w + 8.0, control_y, pause.w, control_h);
        let speeds = std::array::from_fn(|index| {
            Rect::new(
                12.0 + index as f32 * (width - 24.0) / 3.0,
                control_y + control_h + 8.0,
                (width - 24.0) / 3.0 - 6.0,
                control_h,
            )
        });
        let content_y = speeds[0].y + control_h + 12.0;
        let columns = (((width - 8.0) / (widest_label + 16.0)).floor() as usize).clamp(1, 5);
        let row_height = (26.0 * text_scale + 18.0).max(54.0);
        let full_height = 5_usize.div_ceil(columns) as f32 * (row_height + 8.0) - 8.0;
        // Extreme scale/short windows use one persistent navigation opener;
        // destinations then occupy the scrollable reading area as full buttons.
        let compact_navigation = columns < 3 || height - full_height - content_y < 160.0;
        let nav_height = if compact_navigation {
            row_height
        } else {
            full_height
        };
        let navigation = Rect::new(4.0, height - nav_height - 8.0, width - 8.0, nav_height);
        Self {
            title,
            status,
            pause,
            utilities,
            speeds,
            content: Rect::new(
                12.0,
                content_y,
                width - 24.0,
                (navigation.y - content_y - 16.0).max(1.0),
            ),
            navigation,
            compact_navigation,
            columns,
            row_height,
        }
    }

    pub fn destination(&self, index: usize) -> Rect {
        let row = index / self.columns;
        let row_count = (5 - row * self.columns).min(self.columns);
        let width = (self.navigation.w - 8.0 * (row_count - 1) as f32) / row_count as f32;
        Rect::new(
            self.navigation.x + (index % self.columns) as f32 * (width + 8.0),
            self.navigation.y + row as f32 * (self.row_height + 8.0),
            width,
            self.row_height,
        )
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/unit/ui/mobile/layout/tests.rs"
    ));
}
