use super::*;

#[test]
fn phone_shell_keeps_controls_separate_across_accessibility_preferences() {
    for (physical_w, physical_h) in [(390.0, 844.0), (1024.0, 768.0)] {
        for ui_scale in [0.75, 1.0, 1.5, 2.0] {
            for text_scale in [0.75, 1.0, 1.5] {
                let width = physical_w / ui_scale;
                let height = physical_h / ui_scale;
                let l = Layout::new(width, height, text_scale, 56.0 * text_scale);
                assert!(l.title.y >= 0.0);
                assert!(l.status.y >= l.title.y + l.title.h);
                assert!(l.pause.y >= l.status.y + l.status.h);
                assert!(l.content.h >= 60.0, "{width}x{height} / text {text_scale}");
                assert!(l.content.y + l.content.h <= l.navigation.y);
                assert!(l.navigation.y + l.navigation.h <= height);
                if !l.compact_navigation {
                    for i in 0..5 {
                        let cell = l.destination(i);
                        assert!(cell.w >= 56.0 * text_scale + 8.0);
                        assert!(cell.x >= 0.0 && cell.x + cell.w <= width);
                        for j in 0..i {
                            assert!(!cell.overlaps(&l.destination(j)));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn enlarged_phone_labels_gain_a_second_row_and_extreme_scale_gets_an_opener() {
    let regular = Layout::new(390.0, 844.0, 1.0, 56.0);
    assert!(!regular.compact_navigation);
    assert_eq!(regular.destination(0).y, regular.destination(4).y);
    let enlarged = Layout::new(390.0, 844.0, 1.5, 84.0);
    assert!(!enlarged.compact_navigation);
    assert!(enlarged.destination(4).y > enlarged.destination(0).y);
    assert!(Layout::new(195.0, 422.0, 1.5, 84.0).compact_navigation);
}
