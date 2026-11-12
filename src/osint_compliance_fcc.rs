// ## Map
// Role: core/osint_compliance_fcc — Bus 3 慢速反爬蟲 FCC / NCC / 專利情報採集與相依邊推導純函數積木。
// Inputs: FCC / 專利認證中繼資料、保密狀態、頻譜與功率、供應鏈實體代碼。
// Outputs: Bus 3 ChronicleNode / CausalEdge TSV 行字串、供應鏈 target ID、解密破梗評分。
// Invariants: 零全域可變狀態、零外部第三方依賴、無遞迴、控制流巢狀深度 <= 2、100% 符合 09 號合約。

use crate::crypto_vault;

/// 消毒與截斷 Chronicle 節點 slug，移除非英數字元並限制於 64 字元。
/// @outputs: 1
pub fn sanitize_chronicle_slug(raw: &str) -> String {
    let mut out = String::with_capacity(64);
    let mut count = 0usize;
    for ch in raw.chars() {
        if count >= 64 {
            break;
        }
        if ch.is_ascii_alphanumeric() || ch == '-' {
            out.push(ch);
        } else {
            out.push('-');
        }
        count += 1;
    }
    out
}

/// 產出符合 Bus 3 ChronicleNode 標準規範之 TSV 格式行。
/// @outputs: 2
/// @domain: node_id [1..1000000]
/// @domain: known_ts [1..2000000000]
/// @domain: cadence [1..10]
/// @domain: entity [1..1000000]
/// @domain: facets [0..127]
/// @domain: score [1..5]
/// @domain: bottleneck [0..10]
pub fn generate_chronicle_node_tsv(
    node_id: u32,
    known_ts: u64,
    cadence: u8,
    entity: u32,
    facets: u8,
    score: u8,
    bottleneck: u8,
    slug: &str,
) -> Option<String> {
    if node_id == 0 || node_id > 1_000_000 || known_ts == 0 || known_ts > 2_000_000_000 {
        return None;
    }
    if cadence < 1 || cadence > 10 || entity == 0 || entity > 1_000_000 {
        return None;
    }
    if facets > 127 || score < 1 || score > 5 || bottleneck > 10 {
        return None;
    }
    let clean = sanitize_chronicle_slug(slug);
    Some(format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        node_id, known_ts, cadence, entity, facets, score, bottleneck, clean
    ))
}

/// 產出符合 Bus 3 CausalEdge 標準規範之 TSV 格式行。
/// @outputs: 2
/// @domain: edge_id [1..100000000]
/// @domain: from_id [1..1000000]
/// @domain: to_id [1..100000000]
/// @domain: lag_days [0..3650]
/// @domain: score [1..5]
/// @domain: discovered_ts [1..2000000000]
pub fn generate_causal_edge_tsv(
    edge_id: u64,
    from_id: u32,
    to_id: u32,
    lag_days: u16,
    score: u8,
    discovered_ts: u64,
) -> Option<String> {
    if edge_id == 0 || edge_id > 100_000_000 || from_id == 0 || from_id > 1_000_000 {
        return None;
    }
    if to_id == 0 || to_id > 100_000_000 || lag_days > 3650 {
        return None;
    }
    if score < 1 || score > 5 || discovered_ts == 0 || discovered_ts > 2_000_000_000 {
        return None;
    }
    Some(format!(
        "{}\t{}\t{}\t{}\t{}\t{}\n",
        edge_id, from_id, to_id, lag_days, score, discovered_ts
    ))
}

/// 評定 FCC / 專利認證情報價值分數（解密破梗 unsealed 賦予 5 星，保密中賦予 2 星）。
/// @outputs: 2
/// @domain: freq_mhz [0..300000]
/// @domain: power_dbm [-100..100]
pub fn evaluate_fcc_filing_score(is_unsealed: bool, freq_mhz: u32, power_dbm: f32) -> u8 {
    if freq_mhz > 300_000 || power_dbm < -100.0 || power_dbm > 100.0 {
        return 1;
    }
    if is_unsealed {
        5
    } else {
        2
    }
}

/// 自供應鏈代號（如 "2330.TW"）計算確定性 24-bit 目標節點 ID。
/// @outputs: 1
pub fn derive_supplier_target_id(supplier_ticker: &str) -> u32 {
    let hex = crypto_vault::sha256_hex(supplier_ticker.as_bytes());
    let prefix = if hex.len() >= 6 { &hex[..6] } else { &hex };
    u32::from_str_radix(prefix, 16).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @outputs: 1
    #[test]
    fn test_sanitize_slug_alphanumeric_and_truncation() {
        let raw = "nvidia-h100-nvl-unsealed-60ghz-test-report!@#$%^&*()_+";
        let clean = sanitize_chronicle_slug(raw);
        assert_eq!(clean.len(), 54);
        assert!(!clean.contains('!'));
        assert!(!clean.contains('@'));

        let long_raw = "a".repeat(100);
        let truncated = sanitize_chronicle_slug(&long_raw);
        assert_eq!(truncated.len(), 64);
        assert_eq!(truncated, "a".repeat(64));
    }

    /// @outputs: 1
    #[test]
    fn test_generate_node_tsv_golden() {
        let line = generate_chronicle_node_tsv(80001, 1700000000, 2, 1002, 7, 5, 0, "nvidia-h100-nvl");
        assert!(line.is_some());
        // @safety-proof: line is asserted is_some above
        let val = line.unwrap();
        assert_eq!(val, "80001\t1700000000\t2\t1002\t7\t5\t0\tnvidia-h100-nvl\n");

        assert!(generate_chronicle_node_tsv(0, 1700000000, 2, 1002, 7, 5, 0, "slug").is_none());
        assert!(generate_chronicle_node_tsv(1000001, 1700000000, 2, 1002, 7, 5, 0, "slug").is_none());
        assert!(generate_chronicle_node_tsv(80001, 1700000000, 0, 1002, 7, 5, 0, "slug").is_none());
        assert!(generate_chronicle_node_tsv(80001, 1700000000, 2, 1002, 7, 6, 0, "slug").is_none());
    }

    /// @outputs: 1
    #[test]
    fn test_generate_causal_edge_tsv_golden() {
        let edge = generate_causal_edge_tsv(9800011, 80001, 0xABCDEF, 75, 5, 1700000000);
        assert!(edge.is_some());
        // @safety-proof: edge is asserted is_some above
        let val = edge.unwrap();
        assert_eq!(val, "9800011\t80001\t11259375\t75\t5\t1700000000\n");

        assert!(generate_causal_edge_tsv(1, 80001, 100, 3651, 5, 1700000000).is_none());
        assert!(generate_causal_edge_tsv(1, 80001, 100, 75, 0, 1700000000).is_none());
    }

    /// @outputs: 1
    #[test]
    fn test_evaluate_fcc_score_golden() {
        assert_eq!(evaluate_fcc_filing_score(true, 60000, 24.5), 5);
        assert_eq!(evaluate_fcc_filing_score(false, 5850, 19.8), 2);
        assert_eq!(evaluate_fcc_filing_score(true, 300001, 24.5), 1);
        assert_eq!(evaluate_fcc_filing_score(true, 60000, 150.0), 1);
    }

    /// @outputs: 1
    #[test]
    fn test_derive_supplier_target_id_deterministic() {
        let id1 = derive_supplier_target_id("2330.TW");
        let id2 = derive_supplier_target_id("2330.TW");
        assert_eq!(id1, id2);
        assert_ne!(id1, 0);

        let id_foxconn = derive_supplier_target_id("2317.TW");
        assert_ne!(id1, id_foxconn);
    }
}
