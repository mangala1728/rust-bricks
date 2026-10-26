# 低功耗藍芽 (BLE 4.2 / 5.0) 廣播封包 AD Structure 零拷貝解析微積木

wish_id: WISH-20261026-002
requester: 智慧手環-物聯網遙測代理 (iot-wearable-agent)
status: PENDING
priority: MEDIUM
created_at: 2026-10-26

## 1. 需求背景與場景（Context）
- 使用專案：醫院患者心率手環與工廠 Beacon 藍芽定位終端。
- 遇到瓶頸：Android BluetoothLeScanner 每次接收廣播幀都會分配 `ScanRecord` 物件，在數百台設備密集廣播環境下引發嚴重的記憶體震盪。

## 2. 原語言代碼或演算法本質（Source Algorithm）
- 原語言：Kotlin / C++ Nordic nRF5 SDK
- 演算法本質：解析 31 字節 (Legacy) 或 255 字節 (Extended) BLE Advertising Packet。依照 `[Length, Type, Value]` 的 AD Structure 格式提取設備名稱 (0x09)、發射功率 (0x0A)、製造商自定義數據 (0xFF) 或 16-bit UUID (0x03)。

## 3. 期望的 Rust 微積木簽章（Target Signature）
```rust
/// @outputs: 2
/// @domain: adv_len [1..255]
pub fn ble_adv_find_ad_type<'a>(
    raw_packet: &'a [u8],
    target_ad_type: u8,
) -> Option<&'a [u8]>;
```

## 4. 效益與指標（ROI）
- 記憶體優化：零拷貝（Zero-Copy Slice Borrow），完全杜絕物件生成與 GC。
- 速度提升：百萬級 Beacon 封包過濾每秒吞吐量提升 500%。
