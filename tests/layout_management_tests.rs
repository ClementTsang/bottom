//! Mocks layout management, so we can check if we broke anything.

use bottom::app::layout_manager::{BottomLayout, BottomWidgetType};
#[cfg(feature = "battery")]
use bottom::constants::DEFAULT_BATTERY_LAYOUT;
use bottom::constants::{DEFAULT_LAYOUT, DEFAULT_WIDGET_ID};
use bottom::options::{layout_options::Row, Config};
use bottom::utils::error;
use toml_edit::de::from_str;

// TODO: Could move these into the library files rather than external tbh.

const PROC_LAYOUT: &str = r#"
[[row]]
    [[row.child]]
        type="proc"
[[row]]
    [[row.child]]
        type="proc"
    [[row.child]]
        type="proc"
[[row]]
    [[row.child]]
        type="proc"
    [[row.child]]
        type="proc"
"#;

fn test_create_layout(
    rows: &[Row], default_widget_id: u64, default_widget_type: Option<BottomWidgetType>,
    default_widget_count: u64, left_legend: bool,
) -> BottomLayout {
    let mut iter_id = 0; // A lazy way of forcing unique IDs *shrugs*
    let mut total_height_ratio = 0;
    let mut default_widget_count = default_widget_count;
    let mut default_widget_id = default_widget_id;

    let mut ret_bottom_layout = BottomLayout {
        rows: rows
            .iter()
            .map(|row| {
                row.convert_row_to_bottom_row(
                    &mut iter_id,
                    &mut total_height_ratio,
                    &mut default_widget_id,
                    &default_widget_type,
                    &mut default_widget_count,
                    left_legend,
                )
            })
            .collect::<error::Result<Vec<_>>>()
            .unwrap(),
        total_row_height_ratio: total_height_ratio,
    };
    ret_bottom_layout.get_movement_mappings();

    ret_bottom_layout
}

#[test]
/// Tests the default setup.
fn test_default_movement() {
    let rows = from_str::<Config>(DEFAULT_LAYOUT).unwrap().row.unwrap();
    let ret_bottom_layout = test_create_layout(&rows, DEFAULT_WIDGET_ID, None, 1, false);

    // Simple tests for the top CPU widget
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].down_neighbour,
        Some(3)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].right_neighbour,
        Some(2)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].left_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].up_neighbour,
        None
    );

    // Test CPU legend
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].down_neighbour,
        Some(4)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].right_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].left_neighbour,
        Some(1)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].up_neighbour,
        None
    );

    // Test memory->temp, temp->disk, disk->memory mappings
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[0].children[0].right_neighbour,
        Some(4)
    );
    assert_eq!(
        ret_bottom_layout.rows[1].children[1].children[0].children[0].down_neighbour,
        Some(5)
    );
    assert_eq!(
        ret_bottom_layout.rows[1].children[1].children[1].children[0].left_neighbour,
        Some(3)
    );

    // Test disk -> processes, processes -> process sort, process sort -> network
    assert_eq!(
        ret_bottom_layout.rows[1].children[1].children[1].children[0].down_neighbour,
        Some(7)
    );
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[0].children[1].left_neighbour,
        Some(9)
    );
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[0].children[0].left_neighbour,
        Some(6)
    );
}

#[cfg(feature = "battery")]
#[test]
/// Tests battery movement in the default setup.
fn test_default_battery_movement() {
    let rows = from_str::<Config>(DEFAULT_BATTERY_LAYOUT)
        .unwrap()
        .row
        .unwrap();
    let ret_bottom_layout = test_create_layout(&rows, DEFAULT_WIDGET_ID, None, 1, false);

    // Simple tests for the top CPU widget
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].down_neighbour,
        Some(4)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].right_neighbour,
        Some(2)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].left_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].up_neighbour,
        None
    );

    // Test CPU legend
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].down_neighbour,
        Some(5)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].right_neighbour,
        Some(3)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].left_neighbour,
        Some(1)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].up_neighbour,
        None
    );
}

#[test]
/// Tests using left_legend.
fn test_left_legend() {
    let rows = from_str::<Config>(DEFAULT_LAYOUT).unwrap().row.unwrap();
    let ret_bottom_layout = test_create_layout(&rows, DEFAULT_WIDGET_ID, None, 1, true);

    // Legend
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].down_neighbour,
        Some(3)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].right_neighbour,
        Some(1)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].left_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].up_neighbour,
        None
    );

    // Widget
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].down_neighbour,
        Some(3)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].right_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].left_neighbour,
        Some(2)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].up_neighbour,
        None
    );
}

#[test]
/// Tests explicit default widget.
fn test_default_widget_in_layout() {
    let proc_layout = r#"
    [[row]]
        [[row.child]]
            type="proc"
    [[row]]
        [[row.child]]
            type="proc"
        [[row.child]]
            type="proc"
    [[row]]
        [[row.child]]
            type="proc"
            default=true
        [[row.child]]
            type="proc"
    "#;

    let rows = from_str::<Config>(proc_layout).unwrap().row.unwrap();
    let mut iter_id = 0; // A lazy way of forcing unique IDs *shrugs*
    let mut total_height_ratio = 0;
    let mut default_widget_count = 1;
    let mut default_widget_id = DEFAULT_WIDGET_ID;
    let default_widget_type = None;
    let left_legend = false;

    let mut ret_bottom_layout = BottomLayout {
        rows: rows
            .iter()
            .map(|row| {
                row.convert_row_to_bottom_row(
                    &mut iter_id,
                    &mut total_height_ratio,
                    &mut default_widget_id,
                    &default_widget_type,
                    &mut default_widget_count,
                    left_legend,
                )
            })
            .collect::<error::Result<Vec<_>>>()
            .unwrap(),
        total_row_height_ratio: total_height_ratio,
    };
    ret_bottom_layout.get_movement_mappings();

    assert_eq!(default_widget_id, 10);
}

#[test]
/// Tests default widget by setting type and count.
fn test_default_widget_by_option() {
    let rows = from_str::<Config>(PROC_LAYOUT).unwrap().row.unwrap();
    let mut iter_id = 0; // A lazy way of forcing unique IDs *shrugs*
    let mut total_height_ratio = 0;
    let mut default_widget_count = 3;
    let mut default_widget_id = DEFAULT_WIDGET_ID;
    let default_widget_type = Some(BottomWidgetType::Proc);
    let left_legend = false;

    let mut ret_bottom_layout = BottomLayout {
        rows: rows
            .iter()
            .map(|row| {
                row.convert_row_to_bottom_row(
                    &mut iter_id,
                    &mut total_height_ratio,
                    &mut default_widget_id,
                    &default_widget_type,
                    &mut default_widget_count,
                    left_legend,
                )
            })
            .collect::<error::Result<Vec<_>>>()
            .unwrap(),
        total_row_height_ratio: total_height_ratio,
    };
    ret_bottom_layout.get_movement_mappings();

    assert_eq!(default_widget_id, 7);
}

#[test]
fn test_proc_custom_layout() {
    let rows = from_str::<Config>(PROC_LAYOUT).unwrap().row.unwrap();
    let ret_bottom_layout = test_create_layout(&rows, DEFAULT_WIDGET_ID, None, 1, false);

    // First proc widget
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].down_neighbour,
        Some(2)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].left_neighbour,
        Some(3)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].right_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[1].up_neighbour,
        None
    );

    // Its search
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[1].children[0].down_neighbour,
        Some(4)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[1].children[0].left_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[1].children[0].right_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[1].children[0].up_neighbour,
        Some(1)
    );

    // Its sort
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].down_neighbour,
        Some(2)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].left_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].right_neighbour,
        Some(1)
    );
    assert_eq!(
        ret_bottom_layout.rows[0].children[0].children[0].children[0].up_neighbour,
        None
    );

    // Let us now test the second row's first widget...
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[0].children[1].down_neighbour,
        Some(5)
    );
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[0].children[1].left_neighbour,
        Some(6)
    );
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[0].children[1].right_neighbour,
        Some(9)
    );
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[0].children[1].up_neighbour,
        Some(2)
    );

    // Sort
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[0].children[0].down_neighbour,
        Some(5)
    );
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[0].children[0].left_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[0].children[0].right_neighbour,
        Some(4)
    );
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[0].children[0].up_neighbour,
        Some(2)
    );

    // Search
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[1].children[0].down_neighbour,
        Some(10)
    );
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[1].children[0].left_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[1].children[0].right_neighbour,
        Some(8)
    );
    assert_eq!(
        ret_bottom_layout.rows[1].children[0].children[1].children[0].up_neighbour,
        Some(4)
    );

    // Third row, second
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[0].children[1].down_neighbour,
        Some(14)
    );
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[0].children[1].left_neighbour,
        Some(15)
    );
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[0].children[1].right_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[0].children[1].up_neighbour,
        Some(8)
    );

    // Sort
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[0].children[0].down_neighbour,
        Some(14)
    );
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[0].children[0].left_neighbour,
        Some(10)
    );
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[0].children[0].right_neighbour,
        Some(13)
    );
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[0].children[0].up_neighbour,
        Some(8)
    );

    // Search
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[1].children[0].down_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[1].children[0].left_neighbour,
        Some(11)
    );
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[1].children[0].right_neighbour,
        None
    );
    assert_eq!(
        ret_bottom_layout.rows[2].children[1].children[1].children[0].up_neighbour,
        Some(13)
    );
}
