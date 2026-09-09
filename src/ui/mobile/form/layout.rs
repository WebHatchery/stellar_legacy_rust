//! Geometry for the game's flowing section controls and reading footer.
use macroquad::prelude::Rect;

pub(super) struct GroupLayout {
    pub columns: usize,
    pub count: usize,
    pub cell_width: f32,
    pub row_height: f32,
}

impl GroupLayout {
    pub fn new(width: f32, count: usize, max_columns: usize, label_width: f32) -> Self {
        let preferred = (label_width + 24.0).max(80.0);
        let columns = (((width + 12.0) / (preferred + 12.0)).floor() as usize)
            .max(1)
            .min(max_columns.max(1))
            .min(count.max(1));
        Self {
            columns,
            count,
            cell_width: (width - 12.0 * (columns - 1) as f32) / columns as f32,
            row_height: 48.0,
        }
    }

    pub fn height(&self) -> f32 {
        self.count.div_ceil(self.columns) as f32 * (self.row_height + 12.0)
    }

    pub fn cell(&self, area: Rect, index: usize) -> Rect {
        Rect::new(
            area.x + (index % self.columns) as f32 * (self.cell_width + 12.0),
            area.y + (index / self.columns) as f32 * (self.row_height + 12.0),
            self.cell_width,
            self.row_height,
        )
    }
}

pub(super) fn reading_view(area: Rect, content_height: f32, text_scale: f32) -> Rect {
    if content_height <= area.h {
        area
    } else {
        Rect::new(
            area.x,
            area.y,
            area.w,
            (area.h - 28.0 * text_scale).max(1.0),
        )
    }
}

#[cfg(test)]
mod tests;
