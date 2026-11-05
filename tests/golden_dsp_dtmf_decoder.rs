// 本檔由 cloud_farmer 依 specs/dsp_dtmf_decoder.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_dtmf_decoder::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(dtmf_indices_to_char(0, 0) == Some('1'), "spec golden vector #1");
    assert!(dtmf_indices_to_char(3, 1) == Some('0'), "spec golden vector #2");
    assert!(dtmf_indices_to_char(3, 2) == Some('#'), "spec golden vector #3");
    assert!(dtmf_indices_to_char(1, 3) == Some('B'), "spec golden vector #4");
    assert!(dtmf_indices_to_char(4, 0).is_none(), "spec golden vector #5");
    let p = [10.0, 1.0, 1.0, 1.0, 1.0, 12.0, 1.0, 1.0];
    assert!(dtmf_detect_frequencies(&p, 5.0) == Some((0, 1)), "spec golden vector #6");
    let p_low = [1.0, 1.0, 1.0, 1.0, 1.0, 12.0, 1.0, 1.0];
    assert!(dtmf_detect_frequencies(&p_low, 5.0).is_none(), "spec golden vector #7");
}
