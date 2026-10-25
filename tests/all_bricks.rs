// ## Map
// Role: test/all_bricks — 整合與端到端單元測試集。
// Inputs: rust-bricks 公開模組。
// Outputs: 測試通過與斷言驗證。
// Invariants: 零全域可變狀態、覆蓋 100% 函式積木。

use rust_bricks::{audio_dsp, audio_vad, crypto_vault, math_time, sys_path, text_cleaner};

#[test]
fn test_clamp_i64() {
    assert_eq!(math_time::clamp_i64(50, 0, 100), 50);
    assert_eq!(math_time::clamp_i64(-10, 0, 100), 0);
    assert_eq!(math_time::clamp_i64(150, 0, 100), 100);
    assert_eq!(math_time::clamp_i64(50, 100, 0), 50); // 守衛：min > max 拒絕夾止
}

#[test]
fn test_moving_average() {
    let vals = [10.0, 20.0, 30.0];
    assert_eq!(math_time::moving_average_f64(&vals), Some(20.0));
    assert_eq!(math_time::moving_average_f64(&[]), None);
    assert_eq!(math_time::moving_average_f64(&[f64::NAN]), None);
}

#[test]
fn test_julian_day() {
    let jd = math_time::julian_day_from_ymd(2000, 1, 1);
    assert!(jd.is_some());
    // @safety-proof: jd 已驗證 is_some()
    assert!((jd.unwrap() - 2451544.5).abs() < 1.0);
}

#[test]
fn test_dasha_proportion() {
    let prop = math_time::dasha_proportion_days(2191.5, 17.0, 120.0);
    assert!(prop.is_some());
    // @safety-proof: prop 已驗證 is_some()
    assert!(prop.unwrap() > 300.0);
}

#[test]
fn test_pcm16_rms_energy() {
    assert_eq!(audio_vad::pcm16_rms_energy(&[]), 0);
    let silence = vec![0i16; 100];
    assert_eq!(audio_vad::pcm16_rms_energy(&silence), 0);
    let constant_signal = vec![1000i16; 100];
    assert_eq!(audio_vad::pcm16_rms_energy(&constant_signal), 1000);
}

#[test]
fn test_is_silence_frame() {
    assert!(audio_vad::is_silence_frame(50, 100));
    assert!(!audio_vad::is_silence_frame(150, 100));
    assert!(audio_vad::is_silence_frame(40000, 100));
}

#[test]
fn test_cps_chars_per_second() {
    assert_eq!(audio_vad::cps_chars_per_second(10, 2000), Some(5.0));
    assert_eq!(audio_vad::cps_chars_per_second(10, 0), None);
}

#[test]
fn test_constant_time_eq() {
    assert!(crypto_vault::constant_time_eq(b"secret", b"secret"));
    assert!(!crypto_vault::constant_time_eq(b"secret", b"secrxt"));
    assert!(!crypto_vault::constant_time_eq(b"short", b"longer"));
}

#[test]
fn test_sha256_known_vectors() {
    assert_eq!(
        crypto_vault::sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        crypto_vault::sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn test_xor_stream() {
    let text = b"protect me";
    let key = b"my-key";
    let encrypted = crypto_vault::xor_stream(text, key);
    assert_ne!(encrypted, text);
    let decrypted = crypto_vault::xor_stream(&encrypted, key);
    assert_eq!(decrypted, text);
}

#[test]
fn test_is_safe_rel_path() {
    assert!(sys_path::is_safe_rel_path("src/lib.rs"));
    assert!(sys_path::is_safe_rel_path("docs\\guide.md"));
    assert!(!sys_path::is_safe_rel_path("../parent.rs"));
    assert!(!sys_path::is_safe_rel_path("foo/../../bar"));
    assert!(!sys_path::is_safe_rel_path("/etc/passwd"));
    assert!(!sys_path::is_safe_rel_path("C:\\Windows"));
    assert!(!sys_path::is_safe_rel_path(""));
}

#[test]
fn test_normalize_slashes() {
    assert_eq!(sys_path::normalize_slashes("a\\b\\c"), "a/b/c");
    assert_eq!(sys_path::normalize_slashes("a/b/c"), "a/b/c");
}

#[test]
fn test_verify_append_only() {
    let old = "line 1\nline 2\n";
    let append = "line 1\nline 2\nline 3\n";
    assert_eq!(sys_path::verify_append_only(old, append), (true, None));

    let tampered = "line 1\nline 2 modified\nline 3\n";
    assert_eq!(sys_path::verify_append_only(old, tampered), (false, Some(2)));

    let deleted = "line 1\n";
    assert_eq!(sys_path::verify_append_only(old, deleted), (false, Some(2)));
}

#[test]
fn test_truncate_at_boundary() {
    let s = "你好世界";
    assert_eq!(text_cleaner::truncate_at_boundary(s, 4), "你");
    assert_eq!(text_cleaner::truncate_at_boundary(s, 6), "你好");
    assert_eq!(text_cleaner::truncate_at_boundary(s, 20), "你好世界");
}

#[test]
fn test_count_cjk_characters() {
    assert_eq!(text_cleaner::count_cjk_characters("Hello 世界 123"), 2);
    assert_eq!(text_cleaner::count_cjk_characters("Pure English"), 0);
}

#[test]
fn test_audio_dsp_frame_power_and_energy() {
    assert_eq!(audio_dsp::pcm16_frame_power(&[]), 0.0);
    assert_eq!(audio_dsp::energy_dbfs_from_power(0.0), -100.0);
    let full = vec![32767i16; 320];
    let power = audio_dsp::pcm16_frame_power(&full);
    assert!((power - 1.0).abs() < 1e-4);
    assert!((audio_dsp::energy_dbfs_from_power(power) - 0.0).abs() < 1e-3);
}

#[test]
fn test_audio_dsp_snr_subtracted() {
    // Speech power 0.01 (-20 dBFS), noise 0.001 (-30 dBFS) -> SNR ~ 9.54 dB
    let snr = audio_dsp::snr_subtracted_db(0.01, 0.001);
    assert!((snr - 9.542).abs() < 0.01);
    assert_eq!(audio_dsp::snr_subtracted_db(0.05, 0.0), 100.0);
    assert_eq!(audio_dsp::snr_subtracted_db(0.0, 0.0), 100.0);
}

#[test]
fn test_audio_dsp_zero_crossing_rate() {
    assert_eq!(audio_dsp::pcm16_zero_crossing_rate(&[100]), 0.0);
    let alternating = vec![1000, -1000, 1000, -1000, 1000];
    assert_eq!(audio_dsp::pcm16_zero_crossing_rate(&alternating), 1.0);
    let non_crossing = vec![1000, 2000, 3000, 4000];
    assert_eq!(audio_dsp::pcm16_zero_crossing_rate(&non_crossing), 0.0);
}

#[test]
fn test_audio_dsp_biquad_highpass() {
    let coeffs = audio_dsp::biquad_butterworth_highpass_coeffs(150, 16000);
    assert!(coeffs.is_some());
    // @safety-proof: coeffs verified is_some()
    let c = coeffs.unwrap();
    assert_eq!(c.len(), 5);
    assert!((c[0] - c[2]).abs() < 1e-9); // b0 should equal b2
    // Cutoff >= Nyquist should return None
    assert!(audio_dsp::biquad_butterworth_highpass_coeffs(8000, 16000).is_none());
}

#[test]
fn test_audio_dsp_noise_scale_factor() {
    // Target SNR 0 dB with equal power -> scale factor 1.0
    let scale = audio_dsp::mix_noise_scale_factor(0.01, 0.01, 0.0);
    assert!((scale - 1.0).abs() < 1e-4);
    // Invalid powers return 0.0
    assert_eq!(audio_dsp::mix_noise_scale_factor(0.0, 0.01, 10.0), 0.0);
}

#[test]
fn test_math_time_xorshift64() {
    let s1 = math_time::xorshift64_step(12345);
    assert_ne!(s1, 12345);
    let s2 = math_time::xorshift64_step(s1);
    assert_ne!(s2, s1);
    // Zero seed does not produce zero
    let s_zero = math_time::xorshift64_step(0);
    assert_ne!(s_zero, 0);
}

#[test]
fn test_audio_wav_roundtrip() {
    let original: Vec<i16> = vec![0, 1000, -1000, 20000, -20000, 32767, -32768];
    let encoded = rust_bricks::audio_wav::encode_pcm16_wav(&original, 16000);
    assert!(encoded.is_ok());
    // @safety-proof: encoded 已驗證 is_ok()
    let wav_bytes = encoded.unwrap();
    assert_eq!(&wav_bytes[0..4], b"RIFF");
    assert_eq!(&wav_bytes[8..12], b"WAVE");

    let decoded = rust_bricks::audio_wav::decode_pcm16_wav(&wav_bytes);
    assert!(decoded.is_ok());
    // @safety-proof: decoded 已驗證 is_ok()
    let (read_back, rate) = decoded.unwrap();
    assert_eq!(rate, 16000);
    assert_eq!(read_back, original);

    // 異常長度拒絕測試
    assert!(rust_bricks::audio_wav::decode_pcm16_wav(&[0u8; 10]).is_err());
}

#[test]
fn test_sensor_motion_complementary_and_kalman() {
    let angle = rust_bricks::sensor_motion::complementary_filter(0.0, 10.0, 1.0, 0.1, 0.9);
    assert!((angle - 1.0).abs() < 1e-4);

    let (est1, cov1) = rust_bricks::sensor_motion::kalman_filter_1d(0.0, 1.0, 5.0, 0.1, 0.5);
    assert!(est1 > 0.0 && est1 < 5.0);
    assert!(cov1 < 1.1);
}

#[test]
fn test_sensor_motion_haversine() {
    // 台北101 (25.0339, 121.5645) 到 台北車站 (25.0478, 121.5170)
    let dist_opt = rust_bricks::sensor_motion::haversine_distance_m(25.0339, 121.5645, 25.0478, 121.5170);
    assert!(dist_opt.is_some());
    // @safety-proof: dist_opt 已驗證 is_some()
    let dist = dist_opt.unwrap();
    // 距離約 5 公里左右
    assert!(dist > 4500.0 && dist < 5500.0);

    let bearing_opt = rust_bricks::sensor_motion::haversine_bearing_deg(25.0339, 121.5645, 25.0478, 121.5170);
    assert!(bearing_opt.is_some());
    // @safety-proof: bearing_opt 已驗證 is_some()
    let bearing = bearing_opt.unwrap();
    assert!(bearing >= 0.0 && bearing < 360.0);
}

#[test]
fn test_vision_yuv_nv21_and_downsample() {
    let w = 4usize;
    let h = 4usize;
    let mut nv21 = vec![0u8; w * h * 3 / 2];
    for i in 0..(w * h) {
        nv21[i] = (i * 10) as u8;
    }

    let gray_res = rust_bricks::vision_yuv::nv21_extract_grayscale(&nv21, w, h);
    assert!(gray_res.is_ok());
    // @safety-proof: gray_res 已驗證 is_ok()
    let gray = gray_res.unwrap();
    assert_eq!(gray.len(), 16);

    let down_res = rust_bricks::vision_yuv::downsample_2x2_grayscale(&gray, w, h);
    assert!(down_res.is_ok());
    // @safety-proof: down_res 已驗證 is_ok()
    let (down, dw, dh) = down_res.unwrap();
    assert_eq!(dw, 2);
    assert_eq!(dh, 2);
    assert_eq!(down.len(), 4);

    let hist = rust_bricks::vision_yuv::calculate_luminance_histogram(&gray);
    assert_eq!(hist.iter().sum::<u32>(), 16);
}

#[test]
fn test_audio_dsp_pre_emphasis_and_clip() {
    let pcm = vec![1000i16, 2000, 3000, 4000];
    let filtered = rust_bricks::audio_dsp::pre_emphasis_filter(&pcm, 0.97);
    assert_eq!(filtered.len(), 4);
    assert_eq!(filtered[0], 1000);

    // 軟削峰測試
    assert_eq!(rust_bricks::audio_dsp::soft_clip_i16(500, 1000), 500);
    let clipped = rust_bricks::audio_dsp::soft_clip_i16(20000, 10000);
    assert!(clipped < 20000 && clipped > 10000);
}

#[test]
fn test_packet_checksum_crc32_and_adler32_parity() {
    // 1. Python zlib 對齊測試向量
    assert_eq!(rust_bricks::packet_checksum::crc32_ieee(b""), 0x0000_0000);
    assert_eq!(rust_bricks::packet_checksum::crc32_ieee(b"123456789"), 0xcbf4_3926);

    assert_eq!(rust_bricks::packet_checksum::adler32(b""), 0x0000_0001);
    assert_eq!(rust_bricks::packet_checksum::adler32(b"123456789"), 0x091e_01de);

    // 2. 增量更新測試（分段計算 vs 一次性計算）
    let part1 = b"12345";
    let part2 = b"6789";
    let crc_step1 = rust_bricks::packet_checksum::crc32_ieee(part1);
    let crc_full = rust_bricks::packet_checksum::crc32_ieee_update(crc_step1, part2);
    assert_eq!(crc_full, 0xcbf4_3926);

    let adler_step1 = rust_bricks::packet_checksum::adler32(part1);
    let adler_full = rust_bricks::packet_checksum::adler32_update(adler_step1, part2);
    assert_eq!(adler_full, 0x091e_01de);
}

#[test]
fn test_packet_checksum_16bit_mutation_parity() {
    // 3. 8-bit 變 16-bit 異變測試（音訊 PCM16 / 16-bit 感測器數據序列化）
    // 測試陣列：[-1, 0, 1, 32767, -32768]
    // Python struct.pack('<5h', *pcm) 之真值校驗：
    // CRC32 LE: 0xecceca30
    // Adler32 LE: 0x18f603fe
    let pcm = vec![-1i16, 0, 1, 32767, -32768];
    let crc16 = rust_bricks::packet_checksum::crc32_pcm16_le(&pcm);
    assert_eq!(crc16, 0xecce_ca30);

    let adler16 = rust_bricks::packet_checksum::adler32_pcm16_le(&pcm);
    assert_eq!(adler16, 0x18f6_03fe);
}
