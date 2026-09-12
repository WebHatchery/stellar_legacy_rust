use super::*;

#[test]
fn long_section_labels_reflow_without_losing_targets() {
    for width in [155.0, 350.0, 720.0] {
        for label_width in [80.0, 140.0, 210.0] {
            let grid = GroupLayout::new(width, 4, 2, label_width);
            let area = Rect::new(12.0, 172.0, width, grid.height());
            for index in 0..4 {
                let cell = grid.cell(area, index);
                assert!(cell.w >= 44.0 && cell.h >= 44.0);
                assert!(cell.x >= area.x && cell.y >= area.y);
                assert!(cell.x + cell.w <= area.x + area.w + 0.01);
                assert!(cell.y + cell.h <= area.y + area.h);
                for other in 0..index {
                    assert!(!cell.overlaps(&grid.cell(area, other)));
                }
            }
            if label_width + 24.0 > (width - 12.0) / 2.0 {
                assert_eq!(grid.columns, 1);
            }
        }
    }
}

#[test]
fn reading_footer_reserves_scaled_space_only_for_overflow() {
    let area = Rect::new(12.0, 172.0, 350.0, 584.0);
    for scale in [0.75, 1.0, 1.5] {
        assert_eq!(reading_view(area, 580.0, scale), area);
        let view = reading_view(area, 900.0, scale);
        assert_eq!(view.h + 28.0 * scale, area.h);
        assert_eq!(view.y, area.y);
    }
}
