use super::*;

#[test]
fn scale_keeps_dialog_close_and_reset_inside_the_window() {
    for scale in [0.75, 0.95, 1.0, 1.25, 1.5, 2.0] {
        for (width, height) in [(1280.0, 720.0), (1920.0, 1080.0)] {
            let viewport =
                macroquad_toolkit::ui::VirtualUi::from_responsive_screen_size(width, height, scale);
            let l = layout(viewport.logical_width, viewport.logical_height, 1.0);
            assert!(l.panel.x >= 0.0 && l.panel.y >= 0.0);
            assert!(l.panel.right() <= viewport.logical_width);
            assert!(l.panel.bottom() <= viewport.logical_height);
            assert!(macroquad_toolkit::ui::is_fully_visible(l.close, l.panel));
            assert!(macroquad_toolkit::ui::is_fully_visible(
                l.row(0, 0.0),
                l.view
            ));
            assert!(macroquad_toolkit::ui::is_fully_visible(
                l.row(1, 0.0),
                l.view
            ));
        }
    }
}
