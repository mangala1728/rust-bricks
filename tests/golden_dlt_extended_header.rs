// 本檔由 cloud_farmer 依 specs/dlt_extended_header.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dlt_extended_header::*;

#[test]
fn golden_vectors_from_spec() {
    let pkt = [0x41, 0x02, b'A', b'P', b'P', b'1', b'C', b'T', b'X', b'1'];
    let (verb, mtype, level, noar, app, ctx) = dlt_parse_extended_header(&pkt).unwrap();
    assert!(verb == true && mtype == 0 && level == 4 && noar == 2, "spec golden vector #1");
    assert!(&app == b"APP1" && &ctx == b"CTX1", "spec golden vector #2");
    assert!(dlt_parse_extended_header(&[0x41, 0x02, b'A']).is_none(), "spec golden vector #3");
}
