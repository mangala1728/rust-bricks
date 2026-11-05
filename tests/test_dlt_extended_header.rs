// ## Map
// Role: AUTOSAR DLT extended header binary field parser integration test
// Inputs: none (uses hard‑coded golden vectors)
// Outputs: none (asserts)
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::dlt_extended_header::dlt_parse_extended_header;

/// @outputs: 0
#[test]
fn golden_extended_header_parsing() {
    // Golden vector – 10 bytes as defined in the specification
    let pkt = [
        0x41, // MSIN: verbose=1, msg_type=0, log_level=4
        0x02, // NOAR
        b'A', b'P', b'P', b'1', // APP_ID = "APP1"
        b'C', b'T', b'X', b'1', // CONTEXT_ID = "CTX1"
    ];

    let (verb, mtype, level, noar, app, ctx) = dlt_parse_extended_header(&pkt)
        .expect("golden packet must be parsed successfully");

    assert_eq!(verb, true, "verbose flag");
    assert_eq!(mtype, 0, "message type");
    assert_eq!(level, 4, "log level");
    assert_eq!(noar, 2, "number of additional arguments");
    assert_eq!(&app, b"APP1", "application identifier");
    assert_eq!(&ctx, b"CTX1", "context identifier");

    // Buffer shorter than 10 bytes must yield None
    let short_buf = [0x41, 0x02, b'A'];
    assert!(dlt_parse_extended_header(&short_buf).is_none());
}
