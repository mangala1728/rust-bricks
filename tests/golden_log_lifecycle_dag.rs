// 本檔由 cloud_farmer 依 specs/log_lifecycle_dag.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::log_lifecycle_dag::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(resolve_terminal_node(0, &[1, 2, 2], 8) == Some((2, 0)), "spec golden vector #1");
    assert!(resolve_terminal_node(2, &[1, 2, 2], 8) == Some((2, 0)), "spec golden vector #2");
    assert!(resolve_terminal_node(0, &[1, 0], 4) == Some((0, 1)), "spec golden vector #3");
    assert!(classify_epoch_pivot(2, 10, 6, 10, 0) == Some(2), "spec golden vector #4");
    assert!(classify_epoch_pivot(6, 10, 6, 10, 0) == Some(0), "spec golden vector #5");
    assert!(classify_epoch_pivot(7, 10, 6, 10, 0) == Some(0), "spec golden vector #6");
    assert!(classify_epoch_pivot(2, 99, 6, 10, 0) == Some(0), "spec golden vector #7");
    assert!(classify_epoch_pivot(2, 10, 6, 10, 1) == Some(1), "spec golden vector #8");
    assert!(resolve_terminal_node(3, &[0, 1, 2], 8).is_none(), "spec golden vector #9");
    assert!(resolve_terminal_node(0, &[0, 1, 2], 0).is_none(), "spec golden vector #10");
    assert!(classify_epoch_pivot(0, 10, 6, 10, 0).is_none(), "spec golden vector #11");
    assert!(classify_epoch_pivot(2, 10, 6, 10, 2).is_none(), "spec golden vector #12");
}
