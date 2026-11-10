// 本檔由 cloud_farmer 依 specs/ethercat_datagram_header.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::ethercat_datagram_header::*;

#[test]
fn golden_vectors_from_spec() {
    let raw = [0x01, 0x05, 0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00];
    let parsed = ethercat_parse_datagram_header(&raw).unwrap();
    assert!(parsed.cmd == 1, "spec golden vector #1");
    assert!(parsed.idx == 5, "spec golden vector #2");
    assert!(parsed.address == 0x00010000, "spec golden vector #3");
    assert!(parsed.data_len == 4, "spec golden vector #4");
    assert!(parsed.more_follows == false, "spec golden vector #5");
    assert!(parsed.irq == 0, "spec golden vector #6");
    let mut more_raw = raw; more_raw[7] = 0x80;
    let parsed_more = ethercat_parse_datagram_header(&more_raw).unwrap();
    assert!(parsed_more.more_follows == true, "spec golden vector #7");
    assert!(parsed_more.data_len == 4, "spec golden vector #8");
    let mut bad_cmd = raw; bad_cmd[0] = 0;
    assert!(ethercat_parse_datagram_header(&bad_cmd).is_none(), "spec golden vector #9");
    assert!(ethercat_parse_datagram_header(&raw[0..9]).is_none(), "spec golden vector #10");
}
