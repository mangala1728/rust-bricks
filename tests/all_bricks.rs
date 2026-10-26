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

#[test]
fn test_ring_buffer_pcm16_lifecycle() {
    let rb_res = rust_bricks::ring_buffer::RingBufferPcm16::with_capacity(4);
    assert!(rb_res.is_ok());
    // @safety-proof: rb_res 已驗證 is_ok()
    let mut rb = rb_res.unwrap();
    assert_eq!(rb.available_read(), 0);
    assert_eq!(rb.available_write(), 4);

    // 單點 push 與 pop
    assert!(rb.push(100));
    assert!(rb.push(200));
    assert_eq!(rb.available_read(), 2);
    assert_eq!(rb.pop(), Some(100));
    assert_eq!(rb.pop(), Some(200));
    assert_eq!(rb.pop(), None);

    // 塊寫入與塊讀取 (Wrap-around 環繞測試)
    let chunk_in = [1, 2, 3];
    assert_eq!(rb.write_chunk(&chunk_in), 3);
    assert_eq!(rb.available_read(), 3);
    let mut chunk_out = [0i16; 2];
    assert_eq!(rb.read_chunk(&mut chunk_out), 2);
    assert_eq!(chunk_out, [1, 2]);
    assert_eq!(rb.available_read(), 1);

    // 溢出拒絕測試
    let more = [10, 20, 30, 40];
    let written = rb.write_chunk(&more);
    assert_eq!(written, 3); // 只能再寫入 3 格達上限 4
    assert_eq!(rb.available_write(), 0);
    assert!(!rb.push(999));
}

#[test]
fn test_touch_control_bricks() {
    let mut stable_cnt = 0u32;
    // 抖動濾除：單次跳變不應改變狀態（門檻 3）
    assert!(!rust_bricks::touch_control::touch_debounce_filter(false, true, &mut stable_cnt, 3));
    assert!(!rust_bricks::touch_control::touch_debounce_filter(false, true, &mut stable_cnt, 3));
    assert!(rust_bricks::touch_control::touch_debounce_filter(false, true, &mut stable_cnt, 3)); // 滿 3 次確認

    // 速度與平滑
    let (vx, vy, speed) = rust_bricks::touch_control::touch_velocity_estimate(10.0, 0.0, 0.016, 0.5, 0.0, 0.0);
    assert!(vx > 0.0 && vy == 0.0 && speed > 0.0);

    // 壓力分級與遲滯
    assert_eq!(rust_bricks::touch_control::pressure_touch_level(0.05, 0.2, 0.8, 0), 0);
    assert_eq!(rust_bricks::touch_control::pressure_touch_level(0.9, 0.2, 0.8, 0), 2);

    // 手勢狀態機（單擊與雙擊）
    let (s1, ev1) = rust_bricks::touch_control::gesture_tap_state_machine(0, true, 0, 300, 250);
    assert_eq!(s1, 1);
    assert_eq!(ev1, 0);
}

#[test]
fn test_vision_feature_otsu_sobel_integral() {
    // 雙峰直方圖測試 Otsu
    let mut image = vec![30u8; 50];
    image.extend_from_slice(&[200u8; 50]);
    let thresh = rust_bricks::vision_feature::otsu_threshold(&image);
    assert!(thresh >= 30 && thresh <= 200);

    // Sobel 邊緣梯度
    let w = 5usize;
    let h = 5usize;
    let mut edge_img = vec![0u8; w * h];
    for y in 0..h {
        for x in 2..w {
            edge_img[y * w + x] = 255;
        }
    }
    let grad = rust_bricks::vision_feature::sobel_gradient_3x3(&edge_img, w, h, 2, 2);
    assert!(grad > 0);

    // 積分圖與區域求和
    let int_res = rust_bricks::vision_feature::integral_image_compute(&edge_img, w, h);
    assert!(int_res.is_ok());
    // @safety-proof: int_res 已驗證 is_ok()
    let integral = int_res.unwrap();
    let sum_box = rust_bricks::vision_feature::integral_box_sum(&integral, w + 1, 0, 0, w, h);
    assert_eq!(sum_box, (3 * 5 * 255) as u32);
}

#[test]
fn test_bezier_curve_point_and_tangent() {
    // 線性對稱曲線測試：(0, 0) -> (1, 1)
    let p_mid = rust_bricks::bezier_curve::cubic_bezier_point(0.0, 0.333, 0.666, 1.0, 0.5);
    assert!((p_mid - 0.5).abs() < 1e-3);

    let tan = rust_bricks::bezier_curve::cubic_bezier_tangent(0.0, 0.333, 0.666, 1.0, 0.5);
    assert!(tan > 0.0);

    let curv = rust_bricks::bezier_curve::cubic_bezier_curvature([0.0, 0.0, 1.0, 1.0], [0.0, 1.0, 1.0, 0.0], 0.5);
    assert!(curv >= 0.0);
}

#[test]
fn test_sensor_hardware_bricks() {
    // 霍爾皮套翻蓋遲滯測試
    assert!(rust_bricks::sensor_hardware::hall_magnetic_cover_state(150.0, 100.0, 40.0, false));
    assert!(rust_bricks::sensor_hardware::hall_magnetic_cover_state(70.0, 100.0, 40.0, true)); // 保持關閉
    assert!(!rust_bricks::sensor_hardware::hall_magnetic_cover_state(30.0, 100.0, 40.0, true)); // 打開

    // 指南針傾斜補償 (水平放置時)
    let heading_opt = rust_bricks::sensor_hardware::compass_tilt_compensated_bearing(0.0, 0.0, 9.8, 20.0, 0.0, -30.0);
    assert!(heading_opt.is_some());
    // @safety-proof: heading_opt 已驗證 is_some()
    let heading = heading_opt.unwrap();
    assert!(heading >= 0.0 && heading < 360.0);

    // 閃光燈 PWM 與 SOS 信號
    assert!(rust_bricks::sensor_hardware::flashlight_pwm_duty(1000, 50, 200)); // 0..500us 內為 HIGH
    assert!(!rust_bricks::sensor_hardware::flashlight_pwm_duty(1000, 50, 700)); // 500..1000us 內為 LOW
    assert!(rust_bricks::sensor_hardware::sos_morse_signal(0, 100)); // Dit
}

#[test]
fn test_net_telemetry_bricks() {
    // RTT 平滑
    let rtt = rust_bricks::net_telemetry::rtt_jitter_smooth(100.0, 50.0, 0.2);
    assert_eq!(rtt, 60.0);

    // IPv4 CIDR 比對
    assert!(rust_bricks::net_telemetry::ipv4_in_cidr([192, 168, 1, 100], [192, 168, 1, 0], 24));
    assert!(!rust_bricks::net_telemetry::ipv4_in_cidr([192, 168, 2, 100], [192, 168, 1, 0], 24));
    assert!(rust_bricks::net_telemetry::is_private_ipv4([10, 20, 30, 40]));
    assert!(!rust_bricks::net_telemetry::is_private_ipv4([8, 8, 8, 8]));

    // IPv6 CIDR 比對
    let ip6 = [0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    let net6 = [0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    assert!(rust_bricks::net_telemetry::ipv6_in_cidr(ip6, net6, 64));
}

#[test]
fn test_motion_dead_reckon_bricks() {
    // 零速更新 (ZUPT)
    assert!(rust_bricks::motion_dead_reckon::zupt_zero_velocity_detect(0.05, 0.02, 0.1, 0.05));
    assert!(!rust_bricks::motion_dead_reckon::zupt_zero_velocity_detect(1.5, 0.02, 0.1, 0.05));
    assert!(!rust_bricks::motion_dead_reckon::zupt_zero_velocity_detect(-1.0, 0.02, 0.1, 0.05));

    // 1D 航位推算積分
    let (v_stat, p_stat) = rust_bricks::motion_dead_reckon::dead_reckon_1d_step(2.0, 1.0, true, 5.0, 10.0);
    assert_eq!(v_stat, 0.0);
    assert_eq!(p_stat, 10.0);

    let (v_dyn, p_dyn) = rust_bricks::motion_dead_reckon::dead_reckon_1d_step(2.0, 1.0, false, 0.0, 0.0);
    assert!((v_dyn - 2.0).abs() < 1e-5);
    assert!((p_dyn - 1.0).abs() < 1e-5);

    // 馬達振動時序模式
    assert!(rust_bricks::motion_dead_reckon::haptic_vibration_timing(0, 15));
    assert!(!rust_bricks::motion_dead_reckon::haptic_vibration_timing(0, 30));
    assert!(rust_bricks::motion_dead_reckon::haptic_vibration_timing(1, 50));
    assert!(!rust_bricks::motion_dead_reckon::haptic_vibration_timing(1, 150));
    assert!(rust_bricks::motion_dead_reckon::haptic_vibration_timing(1, 250));
    assert!(rust_bricks::motion_dead_reckon::haptic_vibration_timing(2, 20));
    assert!(!rust_bricks::motion_dead_reckon::haptic_vibration_timing(2, 60));
    assert!(rust_bricks::motion_dead_reckon::haptic_vibration_timing(2, 100));
    assert!(rust_bricks::motion_dead_reckon::haptic_vibration_timing(3, 50));
    assert!(!rust_bricks::motion_dead_reckon::haptic_vibration_timing(3, 80));
    assert!(!rust_bricks::motion_dead_reckon::haptic_vibration_timing(99, 10));
}

#[test]
fn test_fast_math_bricks() {
    // 快速倒數平方根 (Quake safe)
    let inv_4 = rust_bricks::fast_math::fast_inv_sqrt_f32(4.0);
    assert!((inv_4 - 0.5).abs() < 0.005);
    let inv_100 = rust_bricks::fast_math::fast_inv_sqrt_f32(100.0);
    assert!((inv_100 - 0.1).abs() < 0.002);
    assert_eq!(rust_bricks::fast_math::fast_inv_sqrt_f32(-1.0), 0.0);

    // 純整數平方根 isqrt_u64
    assert_eq!(rust_bricks::fast_math::isqrt_u64(0), 0);
    assert_eq!(rust_bricks::fast_math::isqrt_u64(1), 1);
    assert_eq!(rust_bricks::fast_math::isqrt_u64(15), 3);
    assert_eq!(rust_bricks::fast_math::isqrt_u64(16), 4);
    assert_eq!(rust_bricks::fast_math::isqrt_u64(1_000_000), 1000);
    assert_eq!(rust_bricks::fast_math::isqrt_u64(u64::MAX), 4294967295);

    // 線性內插 lerp_f32
    assert_eq!(rust_bricks::fast_math::lerp_f32(10.0, 20.0, 0.0), 10.0);
    assert_eq!(rust_bricks::fast_math::lerp_f32(10.0, 20.0, 1.0), 20.0);
    assert_eq!(rust_bricks::fast_math::lerp_f32(10.0, 20.0, 0.5), 15.0);
    assert_eq!(rust_bricks::fast_math::lerp_f32(10.0, 20.0, -0.5), 10.0); // 截斷守衛
    assert_eq!(rust_bricks::fast_math::lerp_f32(10.0, 20.0, 1.5), 20.0);

    // Hermite 平滑階躍 smoothstep_f32
    assert_eq!(rust_bricks::fast_math::smoothstep_f32(0.0, 10.0, -1.0), 0.0);
    assert_eq!(rust_bricks::fast_math::smoothstep_f32(0.0, 10.0, 11.0), 1.0);
    assert!((rust_bricks::fast_math::smoothstep_f32(0.0, 10.0, 5.0) - 0.5).abs() < 1e-5);
    assert_eq!(rust_bricks::fast_math::smoothstep_f32(5.0, 5.0, 5.0), 1.0); // 重合除零防禦
}

#[test]
fn test_geo_collision_bricks() {
    // AABB 碰撞盒重疊
    let box_a_min = (0.0, 0.0);
    let box_a_max = (10.0, 10.0);
    let box_b_min = (5.0, 5.0);
    let box_b_max = (15.0, 15.0);
    let box_c_min = (15.0, 15.0);
    let box_c_max = (20.0, 20.0);

    assert!(rust_bricks::geo_collision::bounding_box_overlap(box_a_min, box_a_max, box_b_min, box_b_max));
    assert!(!rust_bricks::geo_collision::bounding_box_overlap(box_a_min, box_a_max, box_c_min, box_c_max));
    assert!(!rust_bricks::geo_collision::bounding_box_overlap((10.0, 10.0), (0.0, 0.0), box_b_min, box_b_max)); // 倒置

    // 2D 線段交點
    let inter_opt = rust_bricks::geo_collision::segment_intersection_2d(
        (0.0, 0.0), (2.0, 2.0),
        (0.0, 2.0), (2.0, 0.0),
    );
    assert!(inter_opt.is_some());
    // @safety-proof: inter_opt 已驗證 is_some()
    let pt = inter_opt.unwrap();
    assert!((pt.0 - 1.0).abs() < 1e-5);
    assert!((pt.1 - 1.0).abs() < 1e-5);

    // 平行線無交點
    let parallel = rust_bricks::geo_collision::segment_intersection_2d(
        (0.0, 0.0), (2.0, 0.0),
        (0.0, 1.0), (2.0, 1.0),
    );
    assert!(parallel.is_none());

    // 多邊形射線法包含測試 (Jordan Curve)
    let poly = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
    assert!(rust_bricks::geo_collision::point_in_polygon_raycast(5.0, 5.0, &poly));
    assert!(!rust_bricks::geo_collision::point_in_polygon_raycast(15.0, 5.0, &poly));
    assert!(!rust_bricks::geo_collision::point_in_polygon_raycast(-1.0, 5.0, &poly));
    assert!(!rust_bricks::geo_collision::point_in_polygon_raycast(5.0, 5.0, &[])); // 頂點不足
}

#[test]
fn test_iso7816_smartcard_bricks() {
    // 1. Luhn 卡號校驗 (49927398716 為標準真值)
    assert!(rust_bricks::iso7816_smartcard::luhn_checksum_validate(b"49927398716"));
    assert!(!rust_bricks::iso7816_smartcard::luhn_checksum_validate(b"49927398717"));
    assert!(rust_bricks::iso7816_smartcard::luhn_checksum_validate(&[4, 9, 9, 2, 7, 3, 9, 8, 7, 1, 6]));
    assert!(!rust_bricks::iso7816_smartcard::luhn_checksum_validate(b"")); // 空防禦

    // 2. ISO 7816-4 SW1/SW2 狀態分類
    assert_eq!(rust_bricks::iso7816_smartcard::iso7816_sw_status(0x90, 0x00), 0); // 成功
    assert_eq!(rust_bricks::iso7816_smartcard::iso7816_sw_status(0x61, 0x20), 0); // 帶剩餘長度成功
    assert_eq!(rust_bricks::iso7816_smartcard::iso7816_sw_status(0x63, 0xC2), 1); // 警告
    assert_eq!(rust_bricks::iso7816_smartcard::iso7816_sw_status(0x65, 0x81), 2); // 執行錯誤
    assert_eq!(rust_bricks::iso7816_smartcard::iso7816_sw_status(0x6A, 0x82), 3); // 檢查錯誤 (File Not Found)

    // 3. ISO 9564-1 Format 0 PIN Block 構造
    let mut pin_block = [0u8; 8];
    let ok_pin = rust_bricks::iso7816_smartcard::iso9564_pin_block_format0(b"1234", b"1111222233334444", &mut pin_block);
    assert!(ok_pin);
    assert_eq!(pin_block, [0x04, 0x12, 0x26, 0xDD, 0xDC, 0xCC, 0xCB, 0xBB]);

    // 4. 敏感記憶體安全擦除
    let mut secret = [0x5A; 16];
    rust_bricks::iso7816_smartcard::zeroize_slice(&mut secret);
    assert_eq!(secret, [0u8; 16]);

    // 5. EMV BER-TLV Tag/Length 解析 (Tag 0x9F02, Len 6)
    let tlv_stream = [0x9F, 0x02, 0x06, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00];
    let tlv_res = rust_bricks::iso7816_smartcard::ber_tlv_parse_tag_len(&tlv_stream);
    assert!(tlv_res.is_some());
    // @safety-proof: tlv_res 已驗證 is_some()
    let (tag, len, offset) = tlv_res.unwrap();
    assert_eq!(tag, 0x9F02);
    assert_eq!(len, 6);
    assert_eq!(offset, 3);

    // 6. ISO 8583 金融報文點陣圖判定 (0x82 = 10000010b -> Field 1 與 Field 7 存在)
    let bitmap = [0x82, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]; // Field 1, 7, 64
    assert!(rust_bricks::iso7816_smartcard::iso8583_bitmap_is_present(&bitmap, 1));
    assert!(rust_bricks::iso7816_smartcard::iso8583_bitmap_is_present(&bitmap, 7));
    assert!(rust_bricks::iso7816_smartcard::iso8583_bitmap_is_present(&bitmap, 64));
    assert!(!rust_bricks::iso7816_smartcard::iso8583_bitmap_is_present(&bitmap, 2));
    assert!(!rust_bricks::iso7816_smartcard::iso8583_bitmap_is_present(&bitmap, 65)); // 越界
}

#[test]
fn test_can_bus_frame_bricks() {
    // 1. CAN ID 拆解 (標準 11-bit vs 擴展 29-bit)
    let (ext_std, id_std) = rust_bricks::can_bus_frame::can_id_unpack(0x123);
    assert!(!ext_std);
    assert_eq!(id_std, 0x123);

    let (ext_eff, id_eff) = rust_bricks::can_bus_frame::can_id_unpack(0x8000_0000 | 0x18DAF110);
    assert!(ext_eff);
    assert_eq!(id_eff, 0x18DAF110);

    // 2. CAN FD DLC 映射
    assert_eq!(rust_bricks::can_bus_frame::can_fd_dlc_to_len(0), 0);
    assert_eq!(rust_bricks::can_bus_frame::can_fd_dlc_to_len(8), 8);
    assert_eq!(rust_bricks::can_bus_frame::can_fd_dlc_to_len(9), 12);
    assert_eq!(rust_bricks::can_bus_frame::can_fd_dlc_to_len(15), 64);
    assert_eq!(rust_bricks::can_bus_frame::can_fd_dlc_to_len(16), 0); // 守衛超限

    // 3. Modbus RTU CRC16 (多項式 0xA001)
    let modbus_req = [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
    let crc = rust_bricks::can_bus_frame::crc16_modbus(&modbus_req);
    assert_eq!(crc, 0xCDC5);

    // 4. ISO-TP 單幀 (SF) 構造
    let mut sf_frame = [0u8; 8];
    let ok_sf = rust_bricks::can_bus_frame::iso_tp_single_frame_encode(&[0x22, 0xF1, 0x90], &mut sf_frame);
    assert!(ok_sf);
    assert_eq!(sf_frame[0], 0x03);
    assert_eq!(&sf_frame[1..4], &[0x22, 0xF1, 0x90]);
    assert_eq!(&sf_frame[4..8], &[0xCC, 0xCC, 0xCC, 0xCC]);

    // 5. MCU 定時器預分頻與重載週期 (72MHz 產生 1kHz 中斷)
    let timer_res = rust_bricks::can_bus_frame::timer_calc_prescaler_and_period(72_000_000, 1_000);
    assert!(timer_res.is_some());
    // @safety-proof: timer_res 已驗證 is_some()
    let (psc, arr) = timer_res.unwrap();
    assert_eq!((psc as u32 + 1) * (arr as u32 + 1), 72_000);

    // 6. 車載 DBC 跨位元組信號解包 (從 bit 8 開始抽取 16 位元信號)
    // payload: [0xAA, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00] -> byte 1=0x34, byte 2=0x12 -> 0x1234
    let dbc_payload = [0xAA, 0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00];
    let signal = rust_bricks::can_bus_frame::can_signal_unpack_u32(&dbc_payload, 8, 16);
    assert_eq!(signal, 0x1234);
}

#[test]
fn test_spatial_physics_bricks() {
    // 1. 四元數乘法 (繞 Z 軸 90 度 * 90 度 = 180 度)
    let q90_z = (0.7071068_f32, 0.0, 0.0, 0.7071068_f32);
    let q180_z = rust_bricks::spatial_physics::quaternion_multiply(q90_z, q90_z);
    assert!(q180_z.0.abs() < 1e-4);
    assert!((q180_z.3 - 1.0).abs() < 1e-4);

    // 2. 四元數旋轉向量 ((1, 0, 0) 繞 Z 軸轉 90 度 -> (0, 1, 0))
    let v_x = (1.0_f32, 0.0_f32, 0.0_f32);
    let v_rot = rust_bricks::spatial_physics::quaternion_rotate_vector(q90_z, v_x);
    assert!(v_rot.0.abs() < 1e-4);
    assert!((v_rot.1 - 1.0).abs() < 1e-4);
    assert!(v_rot.2.abs() < 1e-4);

    // 3. 射線-球體碰撞 (起點 (0, 0, -5), 方向 (0, 0, 1), 球心 (0, 0, 0), 半徑 2.0 -> 最近交點 t = 3.0)
    let hit_opt = rust_bricks::spatial_physics::ray_sphere_intersect((0.0, 0.0, -5.0), (0.0, 0.0, 1.0), (0.0, 0.0, 0.0), 2.0);
    assert!(hit_opt.is_some());
    // @safety-proof: hit_opt 已驗證 is_some()
    let t_hit = hit_opt.unwrap();
    assert!((t_hit - 3.0).abs() < 1e-4);

    // 4. 彈簧阻尼系統步進 (初始 10.0, 目標 0.0 -> 一步後向 0 移動)
    let (p_next, v_next) = rust_bricks::spatial_physics::spring_damper_step(10.0, 0.0, 0.0, 10.0, 1.0, 0.1);
    assert!(p_next < 10.0);
    assert!(v_next < 0.0);
}

#[test]
fn test_thermal_pid_bricks() {
    // 1. PID 控制步進
    let (out, integral) = rust_bricks::thermal_pid::pid_controller_step(100.0, 20.0, 0.0, 0.0, 2.0, 0.1, 0.5, 0.1, 0.0, 255.0);
    assert!(out > 0.0 && out <= 255.0);
    assert!(integral > 0.0);

    // 2. NTC 測溫 (10k NTC at 25°C)
    let temp_opt = rust_bricks::thermal_pid::ntc_resistance_to_celsius(10000.0, 10000.0, 3950.0, 25.0);
    assert!(temp_opt.is_some());
    // @safety-proof: temp_opt 已驗證 is_some()
    let temp = temp_opt.unwrap();
    assert!((temp - 25.0).abs() < 0.1);

    // 3. 人眼知覺 LED 對數亮度映射
    let duty_half = rust_bricks::thermal_pid::led_perceptual_pwm_duty(128, 255);
    assert!(duty_half < 100); // 人眼平方感知消除低亮跳階

    // 4. 有功電功率計算
    let power = rust_bricks::thermal_pid::electrical_power_watts(5.0, 2.5);
    assert_eq!(power, 12.5);
}

#[test]
fn test_agent_terminal_bricks() {
    // 1. ANSI 脫色 (PTY 日誌純文字淨化)
    let raw_ansi = "\x1b[31;1mError:\x1b[0m disk full";
    let mut clean_buf = [0u8; 64];
    let n = rust_bricks::agent_terminal::strip_ansi_codes(raw_ansi, &mut clean_buf);
    let cleaned = std::str::from_utf8(&clean_buf[..n]).unwrap_or("");
    assert_eq!(cleaned, "Error: disk full");

    // 2. 輕量非遞迴 Glob 匹配
    assert!(rust_bricks::agent_terminal::simple_glob_match("*.rs", "main.rs"));
    assert!(rust_bricks::agent_terminal::simple_glob_match("src/**/*.rs", "src/core/main.rs"));
    assert!(rust_bricks::agent_terminal::simple_glob_match("test_?.rs", "test_1.rs"));
    assert!(!rust_bricks::agent_terminal::simple_glob_match("*.rs", "main.py"));

    // 3. PTY 終端退格模擬
    let raw_input = b"carr\x08go";
    let mut pty_buf = [0u8; 32];
    let pty_len = rust_bricks::agent_terminal::pty_line_buffer_edit(raw_input, &mut pty_buf);
    assert_eq!(&pty_buf[..pty_len], b"cargo");
}

#[test]
fn test_chaos_audit_director() {
    let res = rust_bricks::chaos_audit::run_chaos_audit(0);
    assert!(res.is_ok());
    // @safety-proof: res 已驗證 is_ok()
    let summary = res.unwrap();
    assert_eq!(summary.failed_inspections, 0);
    assert_eq!(summary.total_inspections, summary.passed_inspections);
}

#[test]
fn test_wish_board_listing() {
    let wishes_dir = std::path::Path::new("wishes");
    let res = rust_bricks::wish_board::list_wishes(wishes_dir, 50);
    assert!(res.is_ok());
    // @safety-proof: res 已驗證 is_ok()
    let list = res.unwrap();
    assert!(list.len() >= 2);
}

#[test]
fn test_thermal_palette_bricks() {
    // 1. 鐵紅 (Ironbow) 低溫黑/紫、高溫白
    let (r_low, g_low, b_low) = rust_bricks::thermal_palette::thermal_ironbow_rgb(0.0, 0.0, 100.0);
    assert_eq!((r_low, g_low, b_low), (0, 0, 0));
    let (r_high, g_high, b_high) = rust_bricks::thermal_palette::thermal_ironbow_rgb(100.0, 0.0, 100.0);
    assert_eq!((r_high, g_high, b_high), (255, 255, 255));
    let (r_mid, _, _) = rust_bricks::thermal_palette::thermal_ironbow_rgb(50.0, 0.0, 100.0);
    assert!(r_mid >= 75);

    // 2. 彩虹 (Rainbow) 低溫藍、高溫紅
    let (r_c, g_c, b_c) = rust_bricks::thermal_palette::thermal_rainbow_rgb(0.0, 0.0, 100.0);
    assert_eq!((r_c, g_c, b_c), (0, 0, 255));
    let (r_h, g_h, b_h) = rust_bricks::thermal_palette::thermal_rainbow_rgb(100.0, 0.0, 100.0);
    assert_eq!((r_h, g_h, b_h), (255, 0, 0));

    // 3. 溫度矩陣統計
    let matrix = [20.0, 30.0, 40.0, 10.0];
    let stats_opt = rust_bricks::thermal_palette::thermal_matrix_stats(&matrix);
    assert!(stats_opt.is_some());
    // @safety-proof: stats_opt 已驗證 is_some()
    let (min, max, avg) = stats_opt.unwrap();
    assert_eq!(min, 10.0);
    assert_eq!(max, 40.0);
    assert!((avg - 25.0).abs() < 1e-5);
}

#[test]
fn test_ble_adv_parser_bricks() {
    // 構造標準 BLE 廣播封包：
    // [0x02, 0x0A, 0xEC] -> Len 2, Type 0x0A (TX Power), Val -20 dBm (0xEC = -20)
    // [0x06, 0x09, 'P', 'u', 'l', 's', 'e'] -> Len 6, Type 0x09 (Name), Val "Pulse"
    // [0x04, 0xFF, 0x59, 0x00, 0x01] -> Len 4, Type 0xFF (Mfg), Nordic ID 0x0059, Payload 0x01
    let packet = [
        0x02, 0x0A, 0xEC,
        0x06, 0x09, b'P', b'u', b'l', b's', b'e',
        0x04, 0xFF, 0x59, 0x00, 0x01,
    ];

    // 1. 檢索特定 AD Type
    let ad_type = rust_bricks::ble_adv_parser::ble_adv_find_ad_type(&packet, 0x0A);
    assert_eq!(ad_type, Some(&[0xEC][..]));

    // 2. 解析 TX Power
    let tx_pwr = rust_bricks::ble_adv_parser::ble_adv_parse_tx_power(&packet);
    assert_eq!(tx_pwr, Some(-20));

    // 3. 解析設備名稱
    let name = rust_bricks::ble_adv_parser::ble_adv_parse_device_name(&packet);
    assert_eq!(name, Some("Pulse"));

    // 4. 解析製造商數據
    let mfg = rust_bricks::ble_adv_parser::ble_adv_parse_manufacturer_data(&packet);
    assert!(mfg.is_some());
    // @safety-proof: mfg 已驗證 is_some()
    let (cid, payload) = mfg.unwrap();
    assert_eq!(cid, 0x0059);
    assert_eq!(payload, &[0x01]);
}

#[test]
fn test_dsp_fixed_point_bricks() {
    // 1. Q15 浮點相互轉換
    let q_zero = rust_bricks::dsp_fixed_point::q15_from_f32(0.0);
    assert_eq!(q_zero, 0);
    let q_one = rust_bricks::dsp_fixed_point::q15_from_f32(0.5);
    assert_eq!(q_one, 16384);
    let f_back = rust_bricks::dsp_fixed_point::q15_to_f32(q_one);
    assert!((f_back - 0.5).abs() < 1e-4);

    // 2. Q15 乘法 (0.5 * 0.5 = 0.25 -> 16384 * 16384 = 8192)
    let prod = rust_bricks::dsp_fixed_point::q15_multiply(16384, 16384);
    assert!((prod - 8192).abs() <= 1);

    // 3. FIR 濾波器步進 (均值濾波：兩個 0.5 係數，歷史為 1000, 1000 -> 輸出 1000)
    let coeffs = [16384, 16384]; // 0.5, 0.5
    let history = [1000, 1000];
    let fir_out = rust_bricks::dsp_fixed_point::q15_fir_filter_step(&coeffs, &history);
    assert!((fir_out - 1000).abs() <= 1);

    // 4. Q15 增益調整 (半衰期 * 0.5)
    let mut buf = [1000, 2000, 4000];
    rust_bricks::dsp_fixed_point::q15_apply_gain(&mut buf, 16384);
    assert!((buf[0] - 500).abs() <= 1);
    assert!((buf[1] - 1000).abs() <= 1);
    assert!((buf[2] - 2000).abs() <= 1);
}

#[test]
fn test_crypto_kdf_bricks() {
    // 1. PKCS#7 填充 (10 bytes 補到 16 bytes -> 補 6 個 0x06)
    let data = b"helloworld";
    let mut padded = [0u8; 32];
    let pad_len_opt = rust_bricks::crypto_kdf::pkcs7_pad(data, 16, &mut padded);
    assert_eq!(pad_len_opt, Some(16));
    assert_eq!(&padded[10..16], &[0x06; 6]);

    // 2. PKCS#7 去填充
    let unpad_opt = rust_bricks::crypto_kdf::pkcs7_unpad(&padded[..16], 16);
    assert_eq!(unpad_opt, Some(&b"helloworld"[..]));

    // 剛好滿 16 bytes 時補滿一整塊 16 個 0x10
    let full_block = [0x55u8; 16];
    let mut full_padded = [0u8; 32];
    let full_pad_len = rust_bricks::crypto_kdf::pkcs7_pad(&full_block, 16, &mut full_padded);
    assert_eq!(full_pad_len, Some(32));
    assert_eq!(&full_padded[16..32], &[16u8; 16]);

    // 3. PBKDF 加鹽迭代拉伸
    let mut stretched = [0u8; 16];
    assert!(rust_bricks::crypto_kdf::pbkdf_xor_stretch(b"pin1234", b"salt5678", 1000, &mut stretched));
    assert_ne!(stretched, [0u8; 16]);

    // 4. KDF 衍生工作金鑰
    let master = b"master_bank_key";
    let mut key1 = [0u8; 16];
    let mut key2 = [0u8; 16];
    assert!(rust_bricks::crypto_kdf::simple_kdf_expand(master, b"session_1", &mut key1));
    assert!(rust_bricks::crypto_kdf::simple_kdf_expand(master, b"session_2", &mut key2));
    assert_ne!(key1, key2); // 不同 session 產生的派生金鑰互斥
}

#[test]
fn test_motor_stepper_bricks() {
    // 1. 梯形加減速微秒延遲
    // 總步數 100，加速 20 步，減速 20 步，延遲區間 100us .. 500us
    let delay_start = rust_bricks::motor_stepper::stepper_trapezoid_delay_us(0, 100, 20, 20, 100, 500);
    assert_eq!(delay_start, 500); // 起始最慢
    let delay_cruise = rust_bricks::motor_stepper::stepper_trapezoid_delay_us(50, 100, 20, 20, 100, 500);
    assert_eq!(delay_cruise, 100); // 巡航最高速
    let delay_end = rust_bricks::motor_stepper::stepper_trapezoid_delay_us(99, 100, 20, 20, 100, 500);
    assert!(delay_end > 100); // 減速中

    // 2. 正交編碼器 4x 倍頻步態
    let mut pos = 0i32;
    let s0 = 0b00u8;
    let s1 = rust_bricks::motor_stepper::quadrature_encoder_update(s0, false, true, &mut pos); // 00 -> 01 (+1)
    assert_eq!(pos, 1);
    let s2 = rust_bricks::motor_stepper::quadrature_encoder_update(s1, true, true, &mut pos); // 01 -> 11 (+1)
    assert_eq!(pos, 2);
    let _ = rust_bricks::motor_stepper::quadrature_encoder_update(s2, false, true, &mut pos); // 11 -> 01 (-1)
    assert_eq!(pos, 1);

    // 3. 步數轉行程換算 (200 步/圈，導程 8.0 mm，走 400 步 -> 16.0 mm)
    let dist = rust_bricks::motor_stepper::stepper_steps_to_distance_mm(400, 200, 8.0);
    assert!((dist - 16.0).abs() < 1e-4);
}

#[test]
fn test_crypto_dukpt_bricks() {
    // 1. 解析 KSN 計數器
    // KSN: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x05] -> 計數器為 5
    let mut ksn = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x05];
    assert_eq!(rust_bricks::crypto_dukpt::dukpt_ksn_parse_counter(&ksn), 5);

    // 2. 交易計數器自增
    assert!(rust_bricks::crypto_dukpt::dukpt_ksn_increment_counter(&mut ksn));
    assert_eq!(rust_bricks::crypto_dukpt::dukpt_ksn_parse_counter(&ksn), 6);
    assert_eq!(ksn[9], 0x06);

    // 3. PIN 變體與 Data 變體金鑰衍生
    let base_key = [0x11u8; 16];
    let pin_var = rust_bricks::crypto_dukpt::dukpt_derive_pin_variant(&base_key);
    assert_eq!(pin_var[7], 0x11 ^ 0xFF);
    assert_eq!(pin_var[15], 0x11 ^ 0xFF);
    assert_eq!(pin_var[0], 0x11);

    let data_var = rust_bricks::crypto_dukpt::dukpt_derive_data_variant(&base_key);
    assert_eq!(data_var[5], 0x11 ^ 0xFF);
    assert_eq!(data_var[13], 0x11 ^ 0xFF);
    assert_eq!(data_var[0], 0x11);
}

#[test]
fn test_env_sensor_bricks() {
    // 1. SHT3x CRC8 校驗 (Sensirion 官方驗證向量: 0xBEEF 的 CRC8 為 0x92)
    let crc = rust_bricks::env_sensor::sht3x_crc8(&[0xBE, 0xEF]);
    assert_eq!(crc, 0x92);

    // 2. 露點溫度計算 (25°C, 50% RH -> 露點約 13.8°C ~ 14.0°C)
    let dew_opt = rust_bricks::env_sensor::dew_point_magnus(25.0, 50.0);
    assert!(dew_opt.is_some());
    // @safety-proof: dew_opt 已驗證 is_some()
    let dew = dew_opt.unwrap();
    assert!((dew - 13.9).abs() < 0.5);

    // 100% 濕度時露點等於環境溫度
    let dew_sat = rust_bricks::env_sensor::dew_point_magnus(25.0, 100.0).unwrap();
    // @safety-proof: dew_sat 已在有效域內
    assert!((dew_sat - 25.0).abs() < 0.1);

    // 3. AQI PM2.5 換算
    // 12.0 ug/m3 -> AQI 50
    assert_eq!(rust_bricks::env_sensor::aqi_pm25_us_standard(12.0), 50);
    // 35.4 ug/m3 -> AQI 100
    assert_eq!(rust_bricks::env_sensor::aqi_pm25_us_standard(35.4), 100);
}


