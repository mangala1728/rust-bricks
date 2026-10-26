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

#[test]
fn test_comm_controller_bricks() {
    // 1. I2C 7-bit 地址訊框 (0x3C 寫入 -> 0x78, 讀取 -> 0x79)
    assert_eq!(rust_bricks::comm_controller::i2c_address_frame(0x3C, false), 0x78);
    assert_eq!(rust_bricks::comm_controller::i2c_address_frame(0x3C, true), 0x79);

    // 2. SPI 模式映射 (Mode 0: CPOL=0, CPHA=0; Mode 3: CPOL=1, CPHA=1)
    assert_eq!(rust_bricks::comm_controller::spi_clock_mode(false, false), 0);
    assert_eq!(rust_bricks::comm_controller::spi_clock_mode(true, true), 3);

    // 3. 5 線 SPI D/C 螢幕命令/資料標記
    let (is_cmd, val) = rust_bricks::comm_controller::spi_5wire_dc_frame(true, 0x29); // Display ON
    assert!(is_cmd && val == 0x29);

    // 4. QSPI 0xEB Fast Read 封裝
    let mut q_cmd = [0u8; 8];
    let q_len = rust_bricks::comm_controller::qspi_flash_fast_read_cmd(0x00123456, 6, &mut q_cmd);
    assert_eq!(q_len, 7); // 1 opcode + 3 addr + 3 dummy
    assert_eq!(q_cmd[0], 0xEB);
    assert_eq!(&q_cmd[1..4], &[0x12, 0x34, 0x56]);

    // 5. UART 波特率除數 (16MHz, 115200, 16x 超取樣 -> USARTDIV = 8.6875 -> Mantissa=8, Fraction=11)
    let uart_div = rust_bricks::comm_controller::uart_baud_rate_divisor(16_000_000, 115_200, 16);
    assert!(uart_div.is_some());
    // @safety-proof: uart_div 已驗證 is_some()
    let (mantissa, fraction) = uart_div.unwrap();
    assert_eq!(mantissa, 8);
    assert_eq!(fraction, 11);

    // 6. DMA 跨架構 8/16/32/64 位元對齊
    assert_eq!(rust_bricks::comm_controller::dma_align_padding_bytes(0x1000, 4), 0); // 已對齊
    assert_eq!(rust_bricks::comm_controller::dma_align_padding_bytes(0x1002, 4), 2); // 需補 2 bytes
    assert_eq!(rust_bricks::comm_controller::dma_align_padding_bytes(0x1005, 8), 3); // 需補 3 bytes
}

#[test]
fn test_clock_pll_rtc_bricks() {
    // 1. PLL 分頻倍頻解算 (輸入 8MHz HSE, 目標 72MHz STM32 主頻)
    let pll_opt = rust_bricks::clock_pll_rtc::pll_calculate_prescalers(8_000_000, 72_000_000);
    assert!(pll_opt.is_some());
    // @safety-proof: pll_opt 已驗證 is_some()
    let (m, n, p) = pll_opt.unwrap();
    assert_eq!(m, 8); // 8MHz / 8 = 1MHz ref
    let actual_fout = (1_000_000 * n) / p;
    assert_eq!(actual_fout, 72_000_000);

    // 2. RTC 32.768kHz 晶振拋物線溫漂補償
    // 常溫 25°C 時偏差為 0.0 ppm
    let ppm_25c = rust_bricks::clock_pll_rtc::rtc_32768_ppm_compensate(25.0, 25.0, 0.034);
    assert!(ppm_25c.abs() < 1e-4);
    // 高溫 45°C (delta=20°C) -> ppm = -0.034 * 400 = -13.6 ppm
    let ppm_45c = rust_bricks::clock_pll_rtc::rtc_32768_ppm_compensate(45.0, 25.0, 0.034);
    assert!((ppm_45c - (-13.6)).abs() < 0.1);

    // 3. RTC 補償暫存器換算 (每步 0.954 ppm，-13.6 ppm 需約 14 步)
    let (is_neg, reg_val) = rust_bricks::clock_pll_rtc::rtc_calibration_register_val(ppm_45c, 0.954);
    assert!(is_neg);
    assert_eq!(reg_val, 14);

    // 4. NTP 時戳轉 UNIX 秒數 (NTP 3,900,000,000 -> 3900000000 - 2208988800 = 1691011200)
    let unix_s = rust_bricks::clock_pll_rtc::ntp_timestamp_to_unix_secs(3_900_000_000);
    assert_eq!(unix_s, 1_691_011_200);

    // 5. NTP 雙向往返延遲與時鐘偏移
    // 客戶端發送 t0=1000, 伺服器接收 t1=1020, 伺服器發送 t2=1025, 客戶端接收 t3=1050
    // Delay = (1050 - 1000) - (1025 - 1020) = 50 - 5 = 45 ms
    // Offset = ((1020 - 1000) + (1025 - 1050)) / 2 = (20 + (-25)) / 2 = -2 ms
    let (offset, delay) = rust_bricks::clock_pll_rtc::ntp_clock_offset_and_delay(1000, 1020, 1025, 1050);
    assert_eq!(delay, 45);
    assert_eq!(offset, -2);
}

#[test]
fn test_net_ethernet_bricks() {
    // 1. MAC 廣播與組播判定
    let bcast_mac = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    assert!(rust_bricks::net_ethernet::is_broadcast_mac(&bcast_mac));
    assert!(!rust_bricks::net_ethernet::is_multicast_mac(&bcast_mac));

    let mcast_mac = [0x01, 0x00, 0x5E, 0x00, 0x00, 0xFB]; // mDNS 224.0.0.251
    assert!(rust_bricks::net_ethernet::is_multicast_mac(&mcast_mac));
    assert!(!rust_bricks::net_ethernet::is_broadcast_mac(&mcast_mac));

    let ucast_mac = [0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E];
    assert!(!rust_bricks::net_ethernet::is_broadcast_mac(&ucast_mac));
    assert!(!rust_bricks::net_ethernet::is_multicast_mac(&ucast_mac));

    // 2. 標準 Ethernet II 訊框解析 (IPv4 0x0800)
    let mut std_pkt = [0u8; 64];
    std_pkt[0..6].copy_from_slice(&bcast_mac);
    std_pkt[6..12].copy_from_slice(&ucast_mac);
    std_pkt[12] = 0x08;
    std_pkt[13] = 0x00;
    let eth_info = rust_bricks::net_ethernet::parse_ethernet_frame_header(&std_pkt);
    assert!(eth_info.is_some());
    // @safety-proof: eth_info 已驗證 is_some()
    let info = eth_info.unwrap();
    assert_eq!(info.ether_type, 0x0800);
    assert_eq!(info.vlan_id, None);
    assert_eq!(info.payload_offset, 14);

    // 3. 802.1Q VLAN Tag 訊框解析 (VID=100, EtherType=0x0800)
    let mut vlan_pkt = [0u8; 64];
    vlan_pkt[0..6].copy_from_slice(&bcast_mac);
    vlan_pkt[6..12].copy_from_slice(&ucast_mac);
    vlan_pkt[12] = 0x81;
    vlan_pkt[13] = 0x00; // 802.1Q Tag
    vlan_pkt[14] = 0x00;
    vlan_pkt[15] = 100;  // VID = 100
    vlan_pkt[16] = 0x08;
    vlan_pkt[17] = 0x00; // Inner EtherType IPv4
    let vlan_info = rust_bricks::net_ethernet::parse_ethernet_frame_header(&vlan_pkt).unwrap();
    // @safety-proof: vlan_info 構造保證有效
    assert_eq!(vlan_info.vlan_id, Some(100));
    assert_eq!(vlan_info.ether_type, 0x0800);
    assert_eq!(vlan_info.payload_offset, 18);
}

#[test]
fn test_fin_apdu_track_bricks() {
    // 1. APDU 命令封裝 (SELECT PSE: CLA=0x00, INS=0xA4, P1=0x04, P2=0x00, Data=1PAY.SYS.DDF01)
    let mut apdu_buf = [0u8; 64];
    let aid = b"1PAY.SYS.DDF01";
    let n = rust_bricks::fin_apdu_track::apdu_encode_command(0x00, 0xA4, 0x04, 0x00, aid, Some(0x00), &mut apdu_buf);
    assert!(n.is_some());
    // @safety-proof: n 已驗證 is_some()
    let total_len = n.unwrap();
    assert_eq!(total_len, 4 + 1 + aid.len() + 1);
    assert_eq!(&apdu_buf[0..4], &[0x00, 0xA4, 0x04, 0x00]);
    assert_eq!(apdu_buf[4], aid.len() as u8);
    assert_eq!(apdu_buf[total_len - 1], 0x00); // Le

    // 2. APDU 回應解析
    let resp = [0x6F, 0x1A, 0x84, 0x0E, 0x90, 0x00];
    let parsed = rust_bricks::fin_apdu_track::apdu_parse_response(&resp).unwrap();
    // @safety-proof: parsed 構造保證有效
    assert_eq!(parsed.1, 0x90);
    assert_eq!(parsed.2, 0x00);
    assert_eq!(parsed.0, &[0x6F, 0x1A, 0x84, 0x0E]);

    // 3. 磁條 Track 2 解碼 (PAN=4000123456789010, EXP=2612)
    let raw_t2 = ";4000123456789010=2612201000000?";
    let t2_res = rust_bricks::fin_apdu_track::track2_parse_pan_and_exp(raw_t2);
    assert!(t2_res.is_some());
    // @safety-proof: t2_res 已驗證 is_some()
    let (pan, exp) = t2_res.unwrap();
    assert_eq!(pan, "4000123456789010");
    assert_eq!(exp, "2612");

    // 4. LRC 縱向冗餘校驗
    let lrc = rust_bricks::fin_apdu_track::lrc_checksum(b"12345");
    assert_eq!(lrc, b'1' ^ b'2' ^ b'3' ^ b'4' ^ b'5');
}

#[test]
fn test_sensor_power_meter_bricks() {
    // 1. 歐姆定律分流電流計算 (分流 0.05V, 電阻 0.01 歐姆 -> 5.0A)
    let amps = rust_bricks::sensor_power_meter::shunt_voltage_to_current_amperes(0.05, 0.01);
    assert!((amps - 5.0).abs() < 1e-4);

    // 2. 庫侖計積分 (1.0A 運行 3600 秒 -> 1000.0 mAh)
    let mut accum_mah = 0.0f32;
    rust_bricks::sensor_power_meter::coulomb_counter_integrate_mah(1.0, 3600.0, &mut accum_mah);
    assert!((accum_mah - 1000.0).abs() < 1.0);

    // 3. 鋰電池開路電壓轉 SOC 剩餘電量
    assert_eq!(rust_bricks::sensor_power_meter::lipo_voltage_to_soc_percent(3.20), 0.0);
    assert_eq!(rust_bricks::sensor_power_meter::lipo_voltage_to_soc_percent(4.25), 100.0);
    let soc_mid = rust_bricks::sensor_power_meter::lipo_voltage_to_soc_percent(3.84);
    assert!((soc_mid - 50.0).abs() < 0.1);
}

#[test]
fn test_sensor_baro_als_bricks() {
    // 1. 氣壓高度計 (海平面 1013.25 hPa，當前 1013.25 hPa -> 0 公尺)
    let alt_zero = rust_bricks::sensor_baro_als::baro_pressure_to_altitude_m(1013.25, 1013.25);
    assert!(alt_zero.is_some());
    // @safety-proof: alt_zero 已驗證 is_some()
    assert!(alt_zero.unwrap().abs() < 0.1);

    // 氣壓 900 hPa -> 海拔約 988 公尺
    let alt_900 = rust_bricks::sensor_baro_als::baro_pressure_to_altitude_m(900.0, 1013.25).unwrap();
    // @safety-proof: 氣壓有效
    assert!((alt_900 - 988.0).abs() < 5.0);

    // 2. OPT3001 光照度換算 (E=4, R=100 -> LSB = 0.01 * 16 = 0.16 -> Lux = 16.0)
    let raw_opt = (4u16 << 12) | 100u16;
    let lux_opt = rust_bricks::sensor_baro_als::opt3001_raw_to_lux(raw_opt);
    assert!((lux_opt - 16.0).abs() < 1e-3);

    // 3. BH1750 光照度換算 (Raw=120, MTreg=69 -> 100 Lux)
    let lux_bh = rust_bricks::sensor_baro_als::bh1750_raw_to_lux(120, 69);
    assert!((lux_bh - 100.0).abs() < 0.1);
}

#[test]
fn test_physics_raycast_3d_bricks() {
    // 1. Möller–Trumbore 射線三角形相交
    // 三角形在 XY 平面 Z=5: (0, 0, 5), (2, 0, 5), (0, 2, 5)
    // 射線從 (0.5, 0.5, 0) 朝 +Z 方向 (0, 0, 1) 發射
    let orig = (0.5f32, 0.5f32, 0.0f32);
    let dir = (0.0f32, 0.0f32, 1.0f32);
    let v0 = (0.0f32, 0.0f32, 5.0f32);
    let v1 = (2.0f32, 0.0f32, 5.0f32);
    let v2 = (0.0f32, 2.0f32, 5.0f32);
    let hit_opt = rust_bricks::physics_raycast_3d::ray_triangle_intersect_moller_trumbore(orig, dir, v0, v1, v2);
    assert!(hit_opt.is_some());
    // @safety-proof: hit_opt 已驗證 is_some()
    let hit = hit_opt.unwrap();
    assert!((hit.0 - 0.5).abs() < 1e-4);
    assert!((hit.1 - 0.5).abs() < 1e-4);
    assert!((hit.2 - 5.0).abs() < 1e-4);

    // 2. 向量反射 (入射 (1, -1, 0) 撞擊法線為 (0, 1, 0) 的地面 -> 反射 (1, 1, 0))
    let refl = rust_bricks::physics_raycast_3d::vector3_reflect((1.0, -1.0, 0.0), (0.0, 1.0, 0.0));
    assert!((refl.0 - 1.0).abs() < 1e-4);
    assert!((refl.1 - 1.0).abs() < 1e-4);
    assert!(refl.2.abs() < 1e-4);

    // 3. 點面有號距離
    let dist = rust_bricks::physics_raycast_3d::point_plane_distance_signed((0.0, 5.0, 0.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0));
    assert_eq!(dist, 5.0);
}

#[test]
fn test_can_lin_bus_bricks() {
    // 1. LIN PID 奇偶校驗計算 (Frame ID 0x30 -> 48 -> P0=0, P1=0 -> PID=0x30)
    let pid_30 = rust_bricks::can_lin_bus::lin_calc_pid(0x30);
    assert_eq!(pid_30 & 0x3F, 0x30);

    // Frame ID 0x01 (1): P0 = 1^0^0^0 = 1 (bit 6 = 1), P1 = !(0^0^0^0) = 1 (bit 7 = 1) -> 0xC1
    let pid_01 = rust_bricks::can_lin_bus::lin_calc_pid(0x01);
    assert_eq!(pid_01, 0xC1);

    // 2. LIN 校驗和 (Classic vs Enhanced)
    let data = [0x01, 0x02, 0x03];
    let classic_cs = rust_bricks::can_lin_bus::lin_checksum(None, &data);
    let enhanced_cs = rust_bricks::can_lin_bus::lin_checksum(Some(pid_01), &data);
    assert_ne!(classic_cs, enhanced_cs);

    // 3. I2S 16-bit 左右聲道解壓
    let left_slot = 0x1234_0000u32;
    let right_slot = 0x5678_0000u32;
    let (l, r) = rust_bricks::can_lin_bus::i2s_unpack_stereo_16bit(left_slot, right_slot, false);
    assert_eq!(l, 0x1234);
    assert_eq!(r, 0x5678);
}








#[test]
fn test_net_tsn_macsec_bricks() {
    // 1. IEEE 802.1AS gPTP 事件訊息判別 (Sync=0x0, Follow_Up=0x8)
    assert!(rust_bricks::net_tsn_macsec::ptp_is_event_message(0x0));
    assert!(rust_bricks::net_tsn_macsec::ptp_is_event_message(0x3));
    assert!(!rust_bricks::net_tsn_macsec::ptp_is_event_message(0x8));
    assert!(!rust_bricks::net_tsn_macsec::ptp_is_event_message(0xB));

    // 2. gPTP 標頭解析
    let mut ptp_pkt = [0u8; 44];
    ptp_pkt[0] = 0x10 | 0x02; // Transport=1, MessageType=Pdelay_Req(2)
    ptp_pkt[1] = 0x02;
    ptp_pkt[4] = 0x00;
    ptp_pkt[30] = 0x00;
    ptp_pkt[31] = 0x05; // Sequence ID = 5
    // CorrectionField 設為 2^16 (即 65536) -> 大端序 ptp_pkt[13] = 1, [14] = 0, [15] = 0 -> 代表 1.0 納秒
    ptp_pkt[13] = 0x01;
    let summary_opt = rust_bricks::net_tsn_macsec::ptp_parse_header(&ptp_pkt);
    assert!(summary_opt.is_some());
    // @safety-proof: summary_opt 已驗證 is_some()
    let s = summary_opt.unwrap();
    assert_eq!(s.message_type, 0x02);
    assert_eq!(s.sequence_id, 5);
    assert!((s.correction_ns - 1.0).abs() < 1e-4);

    // 3. Pdelay 鏈路延遲解算
    // t1=1000ns, t2=2000ns, t3=2500ns, t4=3500ns, correction=0.0ns
    // Delay = ((3500 - 1000) - (2500 - 2000) - 0) / 2 = (2500 - 500) / 2 = 1000.0 ns
    let delay_opt = rust_bricks::net_tsn_macsec::ptp_pdelay_mean_path_delay_ns(1000, 2000, 2500, 3500, 0.0);
    assert!(delay_opt.is_some());
    // @safety-proof: delay_opt 已驗證 is_some()
    assert!((delay_opt.unwrap() - 1000.0).abs() < 1e-4);

    // 4. IEEE 802.1AE MACsec SecTAG 解析
    let mut sectag = [0u8; 16];
    sectag[0] = 0x2B; // SC=1, E=1, AN=3
    sectag[1] = 0x00;
    sectag[2] = 0x00;
    sectag[3] = 0x00;
    sectag[4] = 0x00;
    sectag[5] = 0x01; // PN = 1
    let tag_info = rust_bricks::net_tsn_macsec::macsec_parse_sectag(&sectag);
    assert!(tag_info.is_some());
    // @safety-proof: tag_info 已驗證 is_some()
    let info = tag_info.unwrap();
    assert!(info.encryption_enabled);
    assert!(info.has_sci);
    assert_eq!(info.association_number, 3);
    assert_eq!(info.packet_number, 1);
    assert_eq!(info.sectag_len, 16);

    // 5. MACsec 防重放滑動視窗驗證
    assert!(rust_bricks::net_tsn_macsec::macsec_replay_window_validate(100, 101, 32));
    assert!(rust_bricks::net_tsn_macsec::macsec_replay_window_validate(100, 80, 32));
    assert!(!rust_bricks::net_tsn_macsec::macsec_replay_window_validate(100, 50, 32));
    assert!(!rust_bricks::net_tsn_macsec::macsec_replay_window_validate(100, 0, 32));
}


#[test]
fn test_ai_tensor_train_softmax_norm_rope() {
    use rust_bricks::ai_tensor_train;

    // 1. 數值穩定 Softmax (防極端大數溢位)
    let logits = [1000.0, 1001.0, 1002.0];
    let mut probs = [0.0f32; 3];
    assert!(ai_tensor_train::softmax_stable(&logits, &mut probs));
    let sum: f32 = probs.iter().sum();
    assert!((sum - 1.0).abs() < 1e-5);
    assert!(probs[2] > probs[1] && probs[1] > probs[0]);

    // 2. LayerNorm 前向傳播
    let x = [2.0, 4.0, 6.0];
    let gamma = [1.0, 1.0, 1.0];
    let beta = [0.0, 0.0, 0.0];
    let mut ln_out = [0.0f32; 3];
    assert!(ai_tensor_train::layer_norm(&x, &gamma, &beta, 1e-5, &mut ln_out));
    let ln_mean = (ln_out[0] + ln_out[1] + ln_out[2]) / 3.0;
    assert!(ln_mean.abs() < 1e-5);

    // 3. RMSNorm 前向傳播
    let mut rms_out = [0.0f32; 3];
    assert!(ai_tensor_train::rmsnorm_step(&x, &gamma, 1e-5, &mut rms_out));
    assert!(rms_out[0] > 0.0 && rms_out[2] > rms_out[1]);

    // 4. RoPE 旋轉位置編碼 (正交旋轉保模長)
    let pair = (3.0f32, 4.0f32);
    let orig_norm_sq = pair.0 * pair.0 + pair.1 * pair.1;
    let rotated = ai_tensor_train::rope_rotary_embedding_step(pair, 0.785398); // pi / 4
    let rot_norm_sq = rotated.0 * rotated.0 + rotated.1 * rotated.1;
    assert!((orig_norm_sq - rot_norm_sq).abs() < 1e-4);

    // 5. RoPE 頻率角計算
    let theta = ai_tensor_train::rope_frequency_theta(1, 0, 64, 10000.0);
    assert!((theta - 1.0).abs() < 1e-5);
}

#[test]
fn test_ai_tensor_train_adamw_loss_gelu() {
    use rust_bricks::ai_tensor_train;

    // 1. AdamW 優化器步進
    let mut param = 2.0f32;
    let mut m = 0.0f32;
    let mut v = 0.0f32;
    let grad = 0.5f32;
    assert!(ai_tensor_train::adamw_step(&mut param, grad, &mut m, &mut v, 0.01, 0.9, 0.999, 0.01, 1e-8, 1));
    assert!(param < 2.0); // 梯度為正，參數應向下步進

    // 2. 融合交叉熵損失與梯度
    let logits = [1.0, 2.0, 0.5];
    let mut grad_out = [0.0f32; 3];
    let loss_opt = ai_tensor_train::cross_entropy_loss_and_grad(&logits, 1, &mut grad_out);
    assert!(loss_opt.is_some());
    // @safety-proof: loss_opt 已驗證 is_some()
    let loss = loss_opt.unwrap();
    assert!(loss > 0.0);
    let grad_sum: f32 = grad_out.iter().sum();
    assert!(grad_sum.abs() < 1e-5); // Softmax 機率和 1 - 1 = 0

    // 3. GELU 激活函式
    assert_eq!(ai_tensor_train::gelu_forward(0.0), 0.0);
    assert!(ai_tensor_train::gelu_forward(3.0) > 2.9);
    assert!(ai_tensor_train::gelu_forward(-3.0).abs() < 0.05);
}


#[test]
fn test_audio_dsp_biquad_apply() {
    let samples = vec![1000i16; 100];
    let coeffs = [1.0, 0.0, 0.0, 0.0, 0.0]; // 直通濾波 (Pass-through)
    let filtered = rust_bricks::audio_dsp::biquad_filter_apply(&samples, coeffs);
    assert_eq!(filtered.len(), 100);
    assert_eq!(filtered[0], 1000);
}


#[test]
fn test_instrument_scpi_vna_bricks() {
    use rust_bricks::instrument_scpi_vna;

    // 1. SCPI 指令解析
    let cmd = b"MEAS:VOLT:DC?\r\n";
    let parsed = instrument_scpi_vna::scpi_parse_command(cmd);
    assert!(parsed.is_some());
    // @safety-proof: parsed 已驗證 is_some()
    let (name, is_query) = parsed.unwrap();
    assert_eq!(name, b"MEAS:VOLT:DC");
    assert!(is_query);

    // 2. SCPI 數值解析 (科學記號)
    let val_opt = instrument_scpi_vna::scpi_parse_numeric_value(b"-12.5E-3");
    assert!(val_opt.is_some());
    // @safety-proof: val_opt 已驗證 is_some()
    assert!((val_opt.unwrap() - (-0.0125)).abs() < 1e-6);

    // 3. 六位半電表讀值統計
    let readings = [10.00001, 10.00002, 10.00003, 10.00004];
    let stats_opt = instrument_scpi_vna::dmm_reading_stats(&readings);
    assert!(stats_opt.is_some());
    // @safety-proof: stats_opt 已驗證 is_some()
    let (min, max, mean, std) = stats_opt.unwrap();
    assert!((min - 10.00001).abs() < 1e-7);
    assert!((max - 10.00004).abs() < 1e-7);
    assert!((mean - 10.000025).abs() < 1e-7);
    assert!(std > 0.0);

    // 4. VNA S11 與 Smith 圓圖阻抗解算 (50 歐姆匹配點 Gamma=(0, 0) -> Z=(50, 0))
    let (gr, gi) = instrument_scpi_vna::vna_s11_to_complex(-100.0, 0.0);
    assert!(gr.abs() < 1e-4 && gi.abs() < 1e-4);
    let z_opt = instrument_scpi_vna::vna_reflection_to_impedance(0.0, 0.0, 50.0);
    assert!(z_opt.is_some());
    // @safety-proof: z_opt 已驗證 is_some()
    let (r, x) = z_opt.unwrap();
    assert!((r - 50.0).abs() < 1e-4 && x.abs() < 1e-4);

    // 5. VSWR 與 Return Loss
    let vswr_opt = instrument_scpi_vna::vna_calc_vswr_and_return_loss(0.1);
    assert!(vswr_opt.is_some());
    // @safety-proof: vswr_opt 已驗證 is_some()
    let (vswr, rl) = vswr_opt.unwrap();
    assert!((vswr - 1.2222).abs() < 0.01);
    assert!((rl - 20.0).abs() < 0.1);

    // 6. 示波器過零頻率解算 (1kHz 方波取樣率 100kHz)
    let mut wave = Vec::new();
    for i in 0..400 {
        let v = if (i / 50) % 2 == 0 { 2.0f32 } else { -2.0f32 };
        wave.push(v);
    }
    let freq_opt = instrument_scpi_vna::oscilloscope_calc_frequency_hz(&wave, 100_000.0, 0.0, 0.5);
    assert!(freq_opt.is_some());
    // @safety-proof: freq_opt 已驗證 is_some()
    assert!((freq_opt.unwrap() - 1000.0).abs() < 1.0);
}

#[test]
fn test_pcb_cad_fem_bricks() {
    use rust_bricks::pcb_cad_fem;

    // 1. Hammerstad 微帶線阻抗 (標準 50 歐姆 FR4 走線)
    let imp_opt = pcb_cad_fem::microstrip_impedance_hammerstad(3.0, 1.6, 4.4);
    assert!(imp_opt.is_some());
    // @safety-proof: imp_opt 已驗證 is_some()
    let (z0, eps) = imp_opt.unwrap();
    assert!(z0 > 45.0 && z0 < 55.0);
    assert!(eps > 3.0 && eps < 4.0);

    // 2. OpenEMS FDTD CFL 最大時間步長 (1mm 立方網格)
    let dt_opt = pcb_cad_fem::openems_fdtd_cfl_max_timestep_sec(0.001, 0.001, 0.001);
    assert!(dt_opt.is_some());
    // @safety-proof: dt_opt 已驗證 is_some()
    let dt = dt_opt.unwrap();
    assert!(dt > 1.9e-12 && dt < 2.0e-12); // ~1.92 ps

    // 3. Gerber 座標轉微米 (2.4 英吋格式，100000 = 10.0000 inch = 254,000 微米)
    let um_opt = pcb_cad_fem::gerber_coord_to_microns(b"100000", 4);
    assert_eq!(um_opt, Some(254_000));

    // 4. PCB 45 度倒角補償比例
    let frac = pcb_cad_fem::pcb_rf_mitered_bend_fraction(1.0, 1.0);
    assert!(frac.is_some());
    // @safety-proof: frac 已驗證 is_some()
    assert!(frac.unwrap() > 0.5 && frac.unwrap() < 0.9);

    // 5. 2D 多邊形鞋帶公式面積 (4x4 矩形)
    let poly = [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)];
    let area_opt = pcb_cad_fem::polygon_shoelace_area_2d(&poly);
    assert_eq!(area_opt, Some(16.0));
}

#[test]
fn test_motion_render_timeline_bricks() {
    use rust_bricks::motion_render_timeline;

    // 1. Remotion 影格與毫秒互轉 (30 fps)
    let ms = motion_render_timeline::remotion_frame_to_ms(45, 30, 1);
    assert_eq!(ms, 1500);
    let frame = motion_render_timeline::remotion_ms_to_frame(1500, 30, 1);
    assert_eq!(frame, 45);

    // 2. SMPTE 時間碼 (第 1845 幀，30fps -> 00:01:01:15)
    let mut tc = [0u8; 11];
    assert!(motion_render_timeline::smpte_timecode_format(1845, 30, &mut tc));
    assert_eq!(&tc, b"00:01:01:15");

    // 3. CSS Cubic-Bezier 曲線求值 (ease-in-out: 0.42, 0.0, 0.58, 1.0)
    let val_start = motion_render_timeline::css_cubic_bezier_solve(0.42, 0.0, 0.58, 1.0, 0.0);
    let val_end = motion_render_timeline::css_cubic_bezier_solve(0.42, 0.0, 0.58, 1.0, 1.0);
    let val_mid = motion_render_timeline::css_cubic_bezier_solve(0.42, 0.0, 0.58, 1.0, 0.5);
    assert_eq!(val_start, 0.0);
    assert_eq!(val_end, 1.0);
    assert!((val_mid - 0.5).abs() < 0.05);

    // 4. SVG 描邊動畫 dashoffset (長度 200，進度 75% -> 剩餘 50)
    let dash = motion_render_timeline::svg_stroke_dashoffset(200.0, 0.75);
    assert!((dash - 50.0).abs() < 1e-4);

    // 5. FFMPEG 精確剪輯時間字串 (3,661,250 ms -> 01:01:01.250)
    let mut seek_buf = [0u8; 12];
    assert!(motion_render_timeline::ffmpeg_time_seek_str(3_661_250, &mut seek_buf));
    assert_eq!(&seek_buf, b"01:01:01.250");
}


#[test]
fn test_office_calc_layout_bricks() {
    use rust_bricks::office_calc_layout;

    // 1. XLOOKUP 精確查表與近似查表
    let keys = [10.0, 20.0, 30.0, 40.0];
    let vals = [100.0, 200.0, 300.0, 400.0];
    assert_eq!(office_calc_layout::excel_xlookup_f64(20.0, &keys, &vals, true), Some(200.0));
    assert_eq!(office_calc_layout::excel_xlookup_f64(25.0, &keys, &vals, false), Some(200.0)); // 近似匹配小於等於 25 者為 20

    // 2. PMT 貸款等額本息攤還公式 (年息 5% 月息 5%/12，30 年 360 期，10 萬本金 -> 每月 536.82)
    let pmt_opt = office_calc_layout::excel_pmt_loan(0.05 / 12.0, 360, 100_000.0);
    assert!(pmt_opt.is_some());
    // @safety-proof: pmt_opt 已驗證 is_some()
    assert!((pmt_opt.unwrap() - 536.82).abs() < 0.1);

    // 3. NPV 淨現值折現計算 (折現率 10%，現金流 100, 200)
    let npv_opt = office_calc_layout::excel_npv_discounted(0.10, &[110.0, 121.0]);
    assert!(npv_opt.is_some());
    // @safety-proof: npv_opt 已驗證 is_some()
    assert!((npv_opt.unwrap() - 200.0).abs() < 1e-4);

    // 4. 百分位數線性插值 (中位數 50% 與 25% 四分位數)
    let data = [10.0, 20.0, 30.0, 40.0, 50.0];
    assert_eq!(office_calc_layout::excel_percentile_linear(&data, 0.5), Some(30.0));
    assert_eq!(office_calc_layout::excel_percentile_linear(&data, 0.25), Some(20.0));

    // 5. Office 排版單位換算 (Twips 與 EMUs)
    assert_eq!(office_calc_layout::office_units_pt_to_twips(12.0), 240); // 12pt = 240 twips
    assert_eq!(office_calc_layout::office_units_inches_to_emus(1.0), 914_400); // 1 inch = 914,400 EMUs

    // 6. Word 貪婪文字折行 (行寬 100px, 詞寬 40px, 空白 10px -> 40+10+40=90 放得下2個詞，第3個詞換行 -> 2行)
    let lines = office_calc_layout::word_greedy_wrap_line_count(&[40, 40, 40], 100, 10);
    assert_eq!(lines, 2);

    // 7. PPT 元素 1D 均勻等間距對齊佈局 (跨度 50，3 個大小為 10 的形狀 -> 間隔為 10，座標為 0, 20, 40)
    let mut positions = [0.0f32; 3];
    assert!(office_calc_layout::ppt_distribute_elements_1d(&[10.0, 10.0, 10.0], 50.0, &mut positions));
    assert_eq!(positions, [0.0, 20.0, 40.0]);
}

#[test]
fn test_rpa_screen_control_bricks() {
    use rust_bricks::rpa_screen_control;

    // 1. 螢幕顏色感知距離與容差判定
    let c_red = (255, 0, 0);
    let c_near_red = (250, 2, 2);
    assert_eq!(rpa_screen_control::rpa_color_diff_rgb(c_red, c_red), 0);
    assert!(rpa_screen_control::rpa_color_match_tolerance(c_red, c_near_red, 5));
    assert!(!rpa_screen_control::rpa_color_match_tolerance(c_red, (0, 255, 0), 10));

    // 2. 螢幕區域模糊色塊搜尋 (4x4 影像，在 (2, 1) 放紅色像素)
    let mut screen = vec![0u8; 4 * 4 * 3];
    let target_idx = (1 * 4 + 2) * 3;
    screen[target_idx] = 255; // R
    screen[target_idx + 1] = 0;   // G
    screen[target_idx + 2] = 0;   // B
    let found = rpa_screen_control::rpa_screen_find_color_patch(&screen, 4, 4, (255, 0, 0), 2);
    assert_eq!(found, Some((2, 1)));

    // 3. 擬人滑鼠移動軌跡點 (起點 (0,0), 終點 (100,100), 貝茲控制點 (50,0), t=0.5 -> (50, 25))
    let pt = rpa_screen_control::rpa_mouse_human_trajectory_point((0.0, 0.0), (100.0, 100.0), (50.0, 0.0), 0.5);
    assert!((pt.0 - 50.0).abs() < 1e-4 && (pt.1 - 25.0).abs() < 1e-4);

    // 4. 雙擊點擊狀態機判定 (間隔 200ms -> true, 間隔 800ms 超出 300ms 門檻 -> false)
    assert!(rpa_screen_control::rpa_click_double_click_detect(1000, 1200, 300));
    assert!(!rpa_screen_control::rpa_click_double_click_detect(1000, 1800, 300));

    // 5. GPIB IEEE 488.1 3 線交握狀態解碼
    assert_eq!(rpa_screen_control::gpib_ieee488_handshake_step(false, false, false), 0); // Idle
    assert_eq!(rpa_screen_control::gpib_ieee488_handshake_step(false, true, false), 1);  // Listeners ready
    assert_eq!(rpa_screen_control::gpib_ieee488_handshake_step(true, false, false), 2);   // Data valid
    assert_eq!(rpa_screen_control::gpib_ieee488_handshake_step(true, false, true), 3);    // Data accepted

    // 6. RS485 字符傳輸微秒時間 (115200 波特率，8N1 = 10 bits -> 86.8 us -> 86 us)
    let tx_time = rpa_screen_control::rs485_calc_char_tx_time_us(115200, 8, 1, false);
    assert_eq!(tx_time, 86);
}


#[test]
fn test_wishes_audio_and_text_bricks() {
    use rust_bricks::{audio_dsp, text_cleaner};

    // 1. WISH-003: 頻譜噪聲估計與頻譜減法增益計算
    let mut noise_profile = [0.1f32; 256];
    let frame_mag = [0.5f32; 256];
    audio_dsp::update_noise_spectrum(&mut noise_profile, &frame_mag, 0.9);
    assert!((noise_profile[0] - 0.14).abs() < 1e-4);

    let mut gain_out = [0.0f32; 256];
    audio_dsp::compute_spectral_subtraction_gain(&frame_mag, &noise_profile, 1.5, 0.05, &mut gain_out);
    assert!(gain_out[0] > 0.05 && gain_out[0] < 1.0);

    // 2. WISH-004: 多相降取樣 48k -> 16k (3:1)
    let input_48k = vec![1000i16; 300];
    let mut output_16k = [0i16; 100];
    let count = audio_dsp::downsample_3to1_pcm16(&input_48k, &mut output_16k);
    assert_eq!(count, 100);
    assert_eq!(output_16k[0], 1000);

    // 3. WISH-004: AGC 自動增益控制
    let mut quiet_signal = vec![500i16; 100];
    audio_dsp::agc_apply_gain_pcm16(&mut quiet_signal, -12.0, 4.0);
    assert!(quiet_signal[0] > 500);

    // 4. WISH-005: 有界 Levenshtein 編輯距離
    assert_eq!(text_cleaner::levenshtein_distance_bounded("kitten", "sitting", 5), 3);
    assert_eq!(text_cleaner::levenshtein_distance_bounded("hello", "hello", 2), 0);
    assert_eq!(text_cleaner::levenshtein_distance_bounded("abc", "xyz", 1), 1);

    // 5. WISH-005: 前綴模糊匹配
    assert!(text_cleaner::fuzzy_prefix_match("translat", "translation"));
    assert!(text_cleaner::fuzzy_prefix_match("interp", "interpreter"));
    assert!(!text_cleaner::fuzzy_prefix_match("xyz", "apple"));
}


#[test]
fn test_radar_cfar_doppler_bricks() {
    use rust_bricks::radar_cfar_doppler;

    // 1. CA-CFAR 恒虛警率檢測 (背景底噪 1.0，在 index 10 放置 20.0 強目標)
    let mut signal = vec![1.0f32; 25];
    signal[10] = 20.0;
    let mut detections = vec![false; 25];
    let hits = radar_cfar_doppler::radar_ca_cfar_detect_1d(&signal, 1, 2, 3.0, &mut detections);
    assert_eq!(hits, 1);
    assert!(detections[10]);

    // 2. 都卜勒測速 (77GHz 車載雷達，都卜勒頻移 1000Hz -> ~1.95 m/s)
    let vel_opt = radar_cfar_doppler::radar_doppler_velocity_mps(1000.0, 77e9);
    assert!(vel_opt.is_some());
    // @safety-proof: vel_opt 已驗證 is_some()
    assert!((vel_opt.unwrap() - 1.9467).abs() < 0.01);

    // 3. 微都卜勒人體呼吸/心跳胸腔微動位移 (60GHz 雷達，0.1 rad 相位差 -> 0.0398 mm)
    let disp_opt = radar_cfar_doppler::radar_micro_doppler_chest_displacement_mm(0.1, 60.0);
    assert!(disp_opt.is_some());
    // @safety-proof: disp_opt 已驗證 is_some()
    assert!((disp_opt.unwrap() - 0.03976).abs() < 1e-4);

    // 4. 雷達點雲 1D 距離分群 (目標 A 在 ~1m，目標 B 在 ~5m)
    let points = [1.0, 1.2, 5.0, 5.1];
    let mut cluster_ids = [0usize; 4];
    let clusters = radar_cfar_doppler::radar_cluster_points_1d(&points, 0.5, &mut cluster_ids);
    assert_eq!(clusters, 2);
    assert_eq!(cluster_ids, [0, 0, 1, 1]);

    // 5. MIMO 虛擬天線陣列合成 (2 TX * 4 RX -> 8 個虛擬天線通道)
    let tx = [0.0, 4.0];
    let rx = [0.0, 1.0, 2.0, 3.0];
    let mut virt_pos = [0.0f32; 8];
    let count = radar_cfar_doppler::radar_mimo_virtual_array_positions(&tx, &rx, &mut virt_pos);
    assert_eq!(count, 8);
    assert_eq!(virt_pos, [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
}

#[test]
fn test_dsp_fft_radix2_bricks() {
    use rust_bricks::dsp_fft_radix2;

    // 1. 位元反轉置換 (3 bits: 001 -> 100 即 1 -> 4)
    assert_eq!(dsp_fft_radix2::fft_bit_reverse_u32(1, 3), 4);
    assert_eq!(dsp_fft_radix2::fft_bit_reverse_u32(3, 3), 6);

    // 2. FFT 與 IFFT 完整可逆性驗證 (4 點信號 [1.0, 2.0, 3.0, 4.0])
    let orig_real = [1.0f32, 2.0, 3.0, 4.0];
    let mut real = orig_real;
    let mut imag = [0.0f32; 4];

    assert!(dsp_fft_radix2::fft_radix2_f32_inplace(&mut real, &mut imag));
    // DC 分量應為 1 + 2 + 3 + 4 = 10.0
    assert!((real[0] - 10.0).abs() < 1e-4);

    assert!(dsp_fft_radix2::ifft_radix2_f32_inplace(&mut real, &mut imag));
    for i in 0..4 {
        assert!((real[i] - orig_real[i]).abs() < 1e-4);
        assert!(imag[i].abs() < 1e-4);
    }

    // 3. Q15 定點數蝶形運算
    let (a, b) = dsp_fft_radix2::fft_radix2_q15_step(1000, 0, 1000, 0, 32767, 0);
    assert!(a.0 > 0 && b.0 >= 0);
}

#[test]
fn test_decision_mcts_bricks() {
    use rust_bricks::decision_mcts;

    // 1. UCB1 評分公式 (未拜訪節點應為無窮大)
    let score_unvisited = decision_mcts::mcts_ucb1_score(0.0, 0, 100, 1.414);
    assert!(score_unvisited.is_infinite());
    let score_visited = decision_mcts::mcts_ucb1_score(60.0, 100, 500, 1.414);
    assert!(score_visited > 0.6 && score_visited.is_finite());

    // 2. 最佳子節點選取 (子節點 0 未拜訪，應優先選取 0)
    let wins = [0.0, 50.0];
    let visits = [0, 80];
    let best = decision_mcts::mcts_select_best_child(&wins, &visits, 100, 1.414);
    assert_eq!(best, Some(0));

    // 3. 反向傳播更新 (獎勵 1.0 沿路徑 [0, 1] 累加)
    let mut tree_wins = [0.0f32; 4];
    let mut tree_visits = [0u32; 4];
    assert!(decision_mcts::mcts_backpropagate_step(&mut tree_wins, &mut tree_visits, &[0, 2], 1.0));
    assert_eq!(tree_wins[0], 1.0);
    assert_eq!(tree_visits[0], 1);
    assert_eq!(tree_wins[2], 1.0);
    assert_eq!(tree_visits[2], 1);

    // 4. 訪問次數溫度策略映射 (溫度 1.0 正規化比例)
    let node_visits = [10u32, 30, 60];
    let mut policy = [0.0f32; 3];
    assert!(decision_mcts::mcts_visit_count_temperature_policy(&node_visits, 1.0, &mut policy));
    assert!((policy[0] - 0.1).abs() < 1e-4);
    assert!((policy[1] - 0.3).abs() < 1e-4);
    assert!((policy[2] - 0.6).abs() < 1e-4);
}

#[test]
fn test_curve_fit_trend_bricks() {
    use rust_bricks::curve_fit_trend;

    // 1. 線性回歸斜率與截距 (y = 2x + 1)
    let x = [1.0, 2.0, 3.0, 4.0];
    let y = [3.0, 5.0, 7.0, 9.0];
    let (slope, intercept, r) = curve_fit_trend::curve_linear_regression_slope_intercept(&x, &y).unwrap();
    assert!((slope - 2.0).abs() < 1e-4);
    assert!((intercept - 1.0).abs() < 1e-4);
    assert!((r - 1.0).abs() < 1e-4);

    // 2. R² 判定係數 (完美預測 R² = 1.0)
    let y_pred = [3.0, 5.0, 7.0, 9.0];
    let r2 = curve_fit_trend::curve_r_squared_score(&y, &y_pred).unwrap();
    assert!((r2 - 1.0).abs() < 1e-4);

    // 3. Huber 穩健損失 (小誤差二次方，大誤差線性)
    let loss_small = curve_fit_trend::curve_huber_loss_single(0.5, 1.0);
    assert!((loss_small - 0.125).abs() < 1e-4);
    let loss_large = curve_fit_trend::curve_huber_loss_single(2.0, 1.0);
    assert!((loss_large - 1.5).abs() < 1e-4);

    // 4. LOWESS Tricube 核權重與局部平滑
    let w0 = curve_fit_trend::curve_lowess_tricube_weight(0.0);
    assert!((w0 - 1.0).abs() < 1e-4);
    let w_far = curve_fit_trend::curve_lowess_tricube_weight(1.0);
    assert_eq!(w_far, 0.0);
    let smoothed = curve_fit_trend::curve_lowess_smooth_point(2.0, &x, &y, 2.0).unwrap();
    assert!((smoothed - 5.0).abs() < 0.1);

    // 5. RBF 與 Matérn 3/2 核函數
    let rbf_self = curve_fit_trend::curve_kernel_rbf(1.0, 1.0, 1.0, 2.0);
    assert!((rbf_self - 4.0).abs() < 1e-4);
    let matern_self = curve_fit_trend::curve_kernel_matern32(1.0, 1.0, 1.0, 2.0);
    assert!((matern_self - 4.0).abs() < 1e-4);

    // 6. 傅立葉特徵映射
    let mut proj = [0.0f32; 4];
    assert!(curve_fit_trend::curve_fourier_feature_proj(0.0, &[1.0, 2.0], &mut proj));
    assert!((proj[0] - 1.0).abs() < 1e-4 && proj[1].abs() < 1e-4);

    // 7. SIREN 正弦活化與 SiLU 活化
    let siren = curve_fit_trend::curve_siren_sine_activation(0.0, 1.0, 0.0, 30.0);
    assert!(siren.abs() < 1e-4);
    let silu_zero = curve_fit_trend::curve_silu_activation(0.0);
    assert!(silu_zero.abs() < 1e-4);
    let silu_pos = curve_fit_trend::curve_silu_activation(2.0);
    assert!(silu_pos > 1.7 && silu_pos < 1.8);
}

#[test]
fn test_ai_autograd_train_bricks() {
    use rust_bricks::ai_autograd_train;

    // 1. 標量自動求導 (Micrograd 加法與乘法)
    let (ga, gb) = ai_autograd_train::autograd_scalar_add(3.0, 4.0, 2.5);
    assert_eq!(ga, 2.5);
    assert_eq!(gb, 2.5);

    let (ma, mb) = ai_autograd_train::autograd_scalar_mul(3.0, 4.0, 1.0);
    assert_eq!(ma, 4.0); // d(a*b)/da = b
    assert_eq!(mb, 3.0); // d(a*b)/db = a

    // 2. 冪次與 ReLU 求導
    let pow_grad = ai_autograd_train::autograd_scalar_pow(2.0, 3.0, 1.0);
    assert!((pow_grad - 12.0).abs() < 1e-4); // 3 * 2^2 = 12
    let relu_pos = ai_autograd_train::autograd_scalar_relu(2.0, 1.5);
    assert_eq!(relu_pos, 1.5);
    let relu_neg = ai_autograd_train::autograd_scalar_relu(-2.0, 1.5);
    assert_eq!(relu_neg, 0.0);

    // 3. BPE 詞元頻率統計與合併
    let tokens = [10u32, 20, 30, 10, 20, 40];
    let most_freq = ai_autograd_train::bpe_find_most_frequent_pair(&tokens).unwrap();
    assert_eq!(most_freq, (10, 20, 2));

    let mut merged = [0u32; 6];
    let out_len = ai_autograd_train::bpe_merge_pair(&tokens, (10, 20), 99, &mut merged);
    assert_eq!(out_len, 4);
    assert_eq!(&merged[..4], &[99, 30, 99, 40]);

    // 4. GPT-2 因果自注意力遮罩 (Causal Mask)
    let mut scores = [0.0f32; 4]; // 2x2
    assert!(ai_autograd_train::causal_attention_mask_apply(&mut scores, 2, -1e9));
    assert_eq!(scores[0], 0.0);
    assert_eq!(scores[1], -1e9); // row 0, col 1 遮蔽
    assert_eq!(scores[2], 0.0);
    assert_eq!(scores[3], 0.0);

    // 5. 餘弦退火排程 (Cosine Annealing with Warmup)
    let lr_warmup = ai_autograd_train::lr_schedule_cosine_warmup(5, 10, 100, 0.01, 0.001);
    assert!((lr_warmup - 0.0055).abs() < 1e-3);
    let lr_mid = ai_autograd_train::lr_schedule_cosine_warmup(55, 10, 100, 0.01, 0.001);
    assert!(lr_mid > 0.001 && lr_mid < 0.01);
    let lr_end = ai_autograd_train::lr_schedule_cosine_warmup(100, 10, 100, 0.01, 0.001);
    assert_eq!(lr_end, 0.001);

    // 6. 全域梯度 L2 Norm 截斷 (Gradient Clipping)
    let mut grads = [3.0f32, 4.0]; // norm = 5.0
    let norm = ai_autograd_train::gradient_clip_l2_norm(&mut grads, 2.5);
    assert!((norm - 5.0).abs() < 1e-4);
    assert!((grads[0] - 1.5).abs() < 1e-4);
    assert!((grads[1] - 2.0).abs() < 1e-4);

    // 7. 數值穩定 Log-Sum-Exp
    let lse = ai_autograd_train::log_sum_exp_f32(&[1000.0, 1000.0]).unwrap();
    assert!((lse - (1000.0 + std::f32::consts::LN_2)).abs() < 1e-3);
}

#[test]
fn test_comm_phy_bus_bricks() {
    use rust_bricks::comm_phy_bus;

    // 1. MIPI CSI-2 短封包封裝與 6-bit ECC 校驗
    let pkt = comm_phy_bus::mipi_csi2_short_packet_header(0x00, 0, 0x0000); // Frame Start
    assert_eq!(pkt[0], 0x00);
    assert_eq!(pkt[1], 0x00);
    assert_eq!(pkt[2], 0x00);
    assert_eq!(pkt[3], 0x00); // 全零 ECC 應為 0

    let pkt_data = comm_phy_bus::mipi_csi2_short_packet_header(0x12, 1, 0x0400);
    assert_eq!(pkt_data[0], (1 << 6) | 0x12);
    assert_eq!(pkt_data[1], 0x00);
    assert_eq!(pkt_data[2], 0x04);

    // 2. MIPI CSI-2 長封包 CRC16
    let crc = comm_phy_bus::mipi_csi2_packet_crc16(b"123456789");
    assert_ne!(crc, 0);

    // 3. MIPI D-PHY 低功耗狀態解碼
    assert_eq!(comm_phy_bus::mipi_dphy_lp_state_decode(false, false), 0);
    assert_eq!(comm_phy_bus::mipi_dphy_lp_state_decode(false, true), 1);
    assert_eq!(comm_phy_bus::mipi_dphy_lp_state_decode(true, false), 2);
    assert_eq!(comm_phy_bus::mipi_dphy_lp_state_decode(true, true), 3);

    // 4. DSPI 與 QPI 模式訊框
    let dspi = comm_phy_bus::dspi_dual_read_command_frame(0xBB, 0x010203, 4);
    assert_eq!(dspi, [0xBB, 0x01, 0x02, 0x03, 4]);
    assert_eq!(comm_phy_bus::qpi_quad_enable_frame(), [0x38, 0x01]);

    // 5. RGMII 延遲抽頭解算與速率控制位元
    assert_eq!(comm_phy_bus::rgmii_tx_delay_skew_taps(1500, 50), Some(30));
    assert_eq!(comm_phy_bus::rgmii_speed_duplex_control_byte(1000, true), Some(0b1110));
    assert_eq!(comm_phy_bus::rgmii_speed_duplex_control_byte(100, false), Some(0b1001));

    // 6. SGMII 自協商廣告字
    let sgmii = comm_phy_bus::sgmii_auto_negotiation_advertisement(true, true, 2);
    assert_eq!(sgmii & 0x8001, 0x8001); // Link Up (bit 15) + Bit 0

    // 7. 跨 CPU 架構 8-bit 與 32-bit FIFO 記憶體打包與解包
    let raw_bytes = [0x11, 0x22, 0x33, 0x44, 0xAA, 0xBB, 0xCC, 0xDD];
    let mut words_be = [0u32; 2];
    let words_n = comm_phy_bus::bus_pack_8bit_to_32bit_words(&raw_bytes, &mut words_be, true);
    assert_eq!(words_n, 2);
    assert_eq!(words_be[0], 0x11223344);
    assert_eq!(words_be[1], 0xAABBCCDD);

    let mut unpacked_bytes = [0u8; 8];
    let bytes_n = comm_phy_bus::bus_unpack_32bit_to_8bit_bytes(&words_be, &mut unpacked_bytes, true);
    assert_eq!(bytes_n, 8);
    assert_eq!(unpacked_bytes, raw_bytes);
}

#[test]
fn test_radar_biomed_dsp_bricks() {
    use rust_bricks::radar_biomed_dsp;

    // 1. MIMO 正交 Hadamard CDMA 編碼
    let h0 = radar_biomed_dsp::radar_hadamard_cdma_code(0, 4).unwrap();
    let h1 = radar_biomed_dsp::radar_hadamard_cdma_code(1, 4).unwrap();
    assert_eq!(h0, 1);
    assert_eq!(h1, 1);

    // 2. TDMA 發射天線排程
    assert!(radar_biomed_dsp::radar_tdma_tx_burst_active(0, 3, 0));
    assert!(!radar_biomed_dsp::radar_tdma_tx_burst_active(1, 3, 0));
    assert!(radar_biomed_dsp::radar_tdma_tx_burst_active(1, 3, 1));

    // 3. FMCW 距離解析度 (帶寬 1 GHz -> ~0.15 m)
    let res = radar_biomed_dsp::radar_fmcw_range_resolution_meters(1.0e9).unwrap();
    assert!((res - 0.149896).abs() < 1e-4);

    // 4. FMCW 拍頻測距 (fb = 100 kHz, Tc = 1 ms, B = 1 GHz -> R = 15 m)
    let r = radar_biomed_dsp::radar_fmcw_beat_frequency_to_range(100.0e3, 1.0e-3, 1.0e9).unwrap();
    assert!((r - 14.9896).abs() < 1e-3);

    // 5. 微都卜勒人體胸腔微動相位解纏繞 (Phase Unwrap)
    let unwrapped = radar_biomed_dsp::radar_phase_unwrap_step(0.5, 0.4);
    assert!((unwrapped - 0.5).abs() < 1e-4);
    let unwrapped_jump = radar_biomed_dsp::radar_phase_unwrap_step(-3.0, 3.0);
    assert!((unwrapped_jump - (3.0 + 2.0 * std::f32::consts::PI - 6.0)).abs() < 0.5);

    // 6. 生醫呼吸信號帶通濾波
    let mut state = [0.0f32; 2];
    let resp = radar_biomed_dsp::radar_respiration_bandpass_step(1.0, &mut state, 0.9, 0.1);
    assert!(resp > 0.0 && resp < 1.0);

    // 7. 8-bit Q7 與 32-bit Q31 量化與反量化
    let q7_pos = radar_biomed_dsp::dsp_quantize_f32_to_q7(1.0);
    assert_eq!(q7_pos, 127);
    let f_pos = radar_biomed_dsp::dsp_dequantize_q7_to_f32(q7_pos);
    assert!((f_pos - 1.0).abs() < 1e-2);

    let q31_half = radar_biomed_dsp::dsp_quantize_f64_to_q31(0.5);
    let f64_half = radar_biomed_dsp::dsp_dequantize_q31_to_f64(q31_half);
    assert!((f64_half - 0.5).abs() < 1e-6);
}

#[test]
fn test_instrument_industrial_bricks() {
    use rust_bricks::instrument_industrial;

    // 1. SCPI 科學計數法浮點解析
    let v_pos = instrument_industrial::scpi_parse_scientific_float(b"+1.234567E+01\r\n").unwrap();
    assert!((v_pos - 12.34567).abs() < 1e-4);
    let v_neg = instrument_industrial::scpi_parse_scientific_float(b"-5.0e-03").unwrap();
    assert!((v_neg - (-0.005)).abs() < 1e-5);

    // 2. 六位半電表 SCPI 指令格式化
    let mut cmd_buf = [0u8; 32];
    let cmd_len = instrument_industrial::scpi_format_dmm_voltage_range_cmd(10.0, 0.00001, &mut cmd_buf);
    assert!(cmd_len > 0);
    assert_eq!(&cmd_buf[..cmd_len], b"CONF:VOLT:DC 10,DEF\n");

    // 3. CANopen SDO 下載加速訊框與成功回應解析
    let (cob_id, sdo_req) = instrument_industrial::canopen_sdo_download_expedited_req(1, 0x1017, 0x00, &[0xE8, 0x03]).unwrap();
    assert_eq!(cob_id, 0x601);
    assert_eq!(sdo_req[0], 0x2B); // 2 bytes payload: CS = 0x23 | (2 << 2) = 0x2B
    assert_eq!(sdo_req[1], 0x17); // 0x1017 low byte
    assert_eq!(sdo_req[2], 0x10); // 0x1017 high byte
    assert_eq!(sdo_req[3], 0x00);
    assert_eq!(sdo_req[4], 0xE8);
    assert_eq!(sdo_req[5], 0x03);

    let resp_ok = [0x60, 0x17, 0x10, 0x00, 0, 0, 0, 0];
    assert!(instrument_industrial::canopen_sdo_parse_download_response(&resp_ok));

    // 4. CANopen 心跳生產者訊框
    let (hb_cob, hb_data) = instrument_industrial::canopen_heartbeat_producer_frame(5, 0x05);
    assert_eq!(hb_cob, 0x705);
    assert_eq!(hb_data[0], 0x05);

    // 5. Modbus ASCII LRC 計算
    let lrc = instrument_industrial::modbus_ascii_compute_lrc(&[0x01, 0x03, 0x00, 0x00, 0x00, 0x0A]);
    assert_eq!(lrc, 0xF2);

    // 6. Modbus RTU 讀取保持暫存器 (FC 0x03) 訊框構建 (含 CRC16)
    let rtu_frame = instrument_industrial::modbus_rtu_build_read_holding_registers(0x01, 0x0000, 0x0002);
    assert_eq!(rtu_frame[0], 0x01);
    assert_eq!(rtu_frame[1], 0x03);
    assert_eq!(rtu_frame[2], 0x00);
    assert_eq!(rtu_frame[3], 0x00);
    assert_eq!(rtu_frame[4], 0x00);
    assert_eq!(rtu_frame[5], 0x02);
    // CRC 低位先傳、高位後傳
    assert_ne!(rtu_frame[6], 0);

    // 7. Modbus 雙 16-bit 暫存器轉 Float32
    // 0x3F800000 = 1.0f32
    let f_val = instrument_industrial::modbus_decode_float32_registers(0x3F80, 0x0000, false);
    assert_eq!(f_val, 1.0);

    // 8. DSLogic 取樣深度轉時間
    let ms = instrument_industrial::dslogic_sample_depth_to_time_ms(10_000_000, 100_000_000).unwrap();
    assert!((ms - 100.0).abs() < 1e-4);

    // 9. LiteVNA 向量網路分析儀掃頻步進頻率
    let f_step = instrument_industrial::litevna_frequency_sweep_step(50_000, 900_000_000, 101, 50).unwrap();
    assert_eq!(f_step, 50_000 + (900_000_000 - 50_000) / 2);
}

#[test]
fn test_cad_geom_model_bricks() {
    use rust_bricks::cad_geom_model;

    // 1. KiCAD 焊盤孔環寬度 (Annular Ring)
    let ring = cad_geom_model::kicad_via_annular_ring_width_mm(0.3, 0.6).unwrap();
    assert!((ring - 0.15).abs() < 1e-4);

    // 2. KiCAD 走線間距違規判定 (DRC Clearance)
    assert!(cad_geom_model::kicad_trace_clearance_violation((0.0, 0.0), (0.1, 0.0), 0.2));
    assert!(!cad_geom_model::kicad_trace_clearance_violation((0.0, 0.0), (1.0, 0.0), 0.2));

    // 3. IPC-2152 走線溫升解算
    let delta_t = cad_geom_model::ipc2152_trace_temp_rise_celsius(2.0, 100.0, false).unwrap();
    assert!(delta_t > 0.0 && delta_t.is_finite());

    // 4. FreeCAD 圓角弦長
    let chord = cad_geom_model::freecad_fillet_chord_length_mm(10.0, 60.0).unwrap();
    assert!((chord - 10.0).abs() < 1e-4); // 60度等邊三角形弦長 = 半徑

    // 5. FreeCAD 柱體拉伸體積
    let vol = cad_geom_model::freecad_extrude_cylinder_volume_mm3(2.0, 10.0).unwrap();
    assert!((vol - 40.0 * std::f64::consts::PI).abs() < 1e-3);

    // 6. FreeCAD 3D AABB 空間包含判定
    let min_pt = (0.0, 0.0, 0.0);
    let max_pt = (10.0, 10.0, 10.0);
    assert!(cad_geom_model::freecad_bounding_box_3d_containment(min_pt, max_pt, (5.0, 5.0, 5.0)));
    assert!(!cad_geom_model::freecad_bounding_box_3d_containment(min_pt, max_pt, (15.0, 5.0, 5.0)));

    // 7. OpenEMS Mur 吸收邊界係數
    let mur = cad_geom_model::openems_mur_absorbing_boundary_coeff(3.0e8, 1.0e-12, 1.0e-3).unwrap();
    assert!(mur < 0.0 && mur.is_finite());

    // 8. OpenEMS 奈奎斯特網格步長
    let cell_sz = cad_geom_model::openems_mesh_nyquist_cell_size_m(1.0e9, 4.0, 10).unwrap();
    assert!((cell_sz - (0.3 / 2.0 / 10.0)).abs() < 1e-4);

    // 9. SVG 圓弧邊界包圍盒
    let (arc_min, arc_max) = cad_geom_model::svg_path_arc_bounding_box((100.0, 100.0), 50.0).unwrap();
    assert_eq!(arc_min, (50.0, 50.0));
    assert_eq!(arc_max, (150.0, 150.0));

    // 10. Remotion 彈簧阻尼比
    let zeta = cad_geom_model::remotion_spring_damping_ratio(1.0, 100.0, 20.0).unwrap();
    assert!((zeta - 1.0).abs() < 1e-4); // 臨界阻尼
}

#[test]
fn test_spc_six_sigma_msa_bricks() {
    use rust_bricks::spc_six_sigma_msa;

    // 1. 製程潛在能力 Cp (USL=16, LSL=4, sigma=1 -> Cp = 12 / 6 = 2.0)
    let cp = spc_six_sigma_msa::spc_cp_index(16.0, 4.0, 1.0).unwrap();
    assert_eq!(cp, 2.0);

    // 2. 六標準差實際製程能力 Cpk (USL=14, LSL=6, mean=10, sigma=1 -> Cpk = 4 / 3 = 1.3333)
    let (cpk, cpu, cpl) = spc_six_sigma_msa::spc_cpk_index(14.0, 6.0, 10.0, 1.0).unwrap();
    assert!((cpk - 1.3333).abs() < 1e-3);
    assert!((cpu - 1.3333).abs() < 1e-3);
    assert!((cpl - 1.3333).abs() < 1e-3);

    // 偏心製程: mean=11 (偏向 USL) -> Cpk = (14-11)/3 = 1.0
    let (cpk_shift, _, _) = spc_six_sigma_msa::spc_cpk_index(14.0, 6.0, 11.0, 1.0).unwrap();
    assert_eq!(cpk_shift, 1.0);

    // 3. 常態分佈累積概率 CDF Phi(0) = 0.5, Phi(1.96) ~ 0.975
    let phi_0 = spc_six_sigma_msa::spc_normal_cdf_phi(0.0);
    assert!((phi_0 - 0.5).abs() < 1e-4);
    let phi_196 = spc_six_sigma_msa::spc_normal_cdf_phi(1.96);
    assert!((phi_196 - 0.975).abs() < 1e-3);

    // 4. 規格內概率與不良率 DPPM (4-sigma 雙邊不良率約 63 PPM)
    let (in_spec, dppm) = spc_six_sigma_msa::spc_spec_in_boundary_probability(14.0, 6.0, 10.0, 1.0).unwrap();
    assert!(in_spec > 0.9999);
    assert!(dppm < 70.0 && dppm > 50.0);

    // 5. 截斷常態分佈統計值 (對稱截斷 [-2, 2], mean=0 -> 截斷均值恆為 0, 方差縮小)
    let (t_mean, t_std, shift) = spc_six_sigma_msa::spc_truncated_normal_stats(-2.0, 2.0, 0.0, 1.0).unwrap();
    assert!(t_mean.abs() < 1e-5);
    assert!(shift.abs() < 1e-5);
    assert!(t_std < 1.0 && t_std > 0.8);

    // 6. 截斷機率密度函數 PDF
    let pdf_center = spc_six_sigma_msa::spc_truncated_normal_pdf(0.0, -2.0, 2.0, 0.0, 1.0).unwrap();
    assert!(pdf_center > 0.39);

    // 7. AIAG 車用 MSA: Gage R&R = sqrt(EV^2 + AV^2) (EV=3, AV=4 -> GRR=5)
    let grr = spc_six_sigma_msa::msa_gage_rr_ev_av(3.0, 4.0).unwrap();
    assert_eq!(grr, 5.0);

    // 8. %GRR (GRR=5, PV=12 -> TV=13 -> %GRR = 5/13 = 38.46%)
    let pct_grr = spc_six_sigma_msa::msa_gage_rr_percent_grr(5.0, 12.0).unwrap();
    assert!((pct_grr - (5.0 / 13.0 * 100.0)).abs() < 1e-4);

    // 9. Precision-to-Tolerance (P/T Ratio, GRR=1, Tol=30 -> 6*1/30 = 20% < 30% 合格)
    let pt = spc_six_sigma_msa::msa_precision_to_tolerance_ratio(1.0, 130.0, 100.0).unwrap();
    assert_eq!(pt, 20.0);

    // 10. 零件有效區分度分類數 ndc = floor(1.414 * PV / GRR) (PV=10, GRR=2 -> ndc=7 >= 5 合格)
    let ndc = spc_six_sigma_msa::msa_number_of_distinct_categories_ndc(10.0, 2.0).unwrap();
    assert_eq!(ndc, 7);
}

#[test]
fn test_quant_finance_market_bricks() {
    use rust_bricks::quant_finance_market;

    // 1. 技術面 EMA 步進 (alpha = 2 / (9+1) = 0.2, EMA = 0.2 * 12 + 0.8 * 10 = 10.4)
    let ema = quant_finance_market::quant_ema_step(10.0, 12.0, 9).unwrap();
    assert!((ema - 10.4).abs() < 1e-4);

    // 2. 技術面 RSI (RS = 3 / 1 = 3 -> RSI = 100 - 100/4 = 75.0)
    let rsi = quant_finance_market::quant_rsi_step(3.0, 1.0).unwrap();
    assert_eq!(rsi, 75.0);

    // 3. 技術面布林通道 (Bollinger Bands)
    let prices = [10.0, 12.0, 14.0, 16.0, 18.0];
    let (upper, sma, lower) = quant_finance_market::quant_bollinger_bands(&prices, 2.0).unwrap();
    assert_eq!(sma, 14.0);
    assert!(upper > sma && lower < sma);

    // 4. MACD 差離值
    let macd = quant_finance_market::quant_macd_line(15.5, 14.0);
    assert!((macd - 1.5).abs() < 1e-4);

    // 5. 基本面 PE 與 PB 比率 (Price=100, EPS=5, BPS=50 -> PE=20, PB=2)
    let (pe, pb) = quant_finance_market::quant_pe_pb_ratio(100.0, 5.0, 50.0).unwrap();
    assert_eq!(pe, 20.0);
    assert_eq!(pb, 2.0);

    // 6. 基本面 ROE 與 FCF Yield
    let (roe, fcf_yield) = quant_finance_market::quant_roe_and_fcf_yield(20.0, 100.0, 8.0, 100.0).unwrap();
    assert_eq!(roe, 20.0);
    assert_eq!(fcf_yield, 8.0);

    // 7. 籌碼面 VWAP 累計步進
    let (cum_vp, cum_vol, vwap) = quant_finance_market::quant_vwap_step(1000.0, 10.0, 110.0, 10.0).unwrap();
    assert_eq!(cum_vp, 2100.0);
    assert_eq!(cum_vol, 20.0);
    assert_eq!(vwap, 105.0);

    // 8. 籌碼面 大單買賣淨流入率 (Buy=600, Sell=400 -> Net = +20%)
    let net_ratio = quant_finance_market::quant_order_flow_net_inflow_ratio(600.0, 400.0).unwrap();
    assert_eq!(net_ratio, 20.0);

    // 9. 期貨基差與正逆價差判定 (Contango vs Backwardation)
    let (basis, is_contango) = quant_finance_market::quant_futures_basis_spread(20100.0, 20000.0);
    assert_eq!(basis, 100.0);
    assert!(is_contango);

    // 10. 期貨年化持有成本殖利率 (Basis=100, Spot=20000, DTE=36.5 -> 0.5% * 10 = 5.0%)
    let carry = quant_finance_market::quant_futures_annualized_carry_yield(20100.0, 20000.0, 36.5).unwrap();
    assert!((carry - 5.0).abs() < 1e-4);

    // 11. 匯市調整後報酬 (美股報酬 10%, 美元對台幣從 30 升至 31.5 -> 總報酬 = 1.1 * 1.05 - 1 = 15.5%)
    let fx_ret = quant_finance_market::quant_fx_adjusted_return(0.10, 30.0, 31.5).unwrap();
    assert!((fx_ret - 0.155).abs() < 1e-4);
}

#[test]
fn test_robotics_motion_profile_bricks() {
    use rust_bricks::robotics_motion_profile;

    // 1. 2-Link 平面機械臂逆向運動學 (IK) 與正向驗證 (FK)
    let (t1, t2) = robotics_motion_profile::robotics_planar_2link_ik(1.0, 1.0, 1.0, 1.0, false).unwrap();
    let (fk_x, fk_y) = robotics_motion_profile::robotics_planar_2link_fk(t1, t2, 1.0, 1.0).unwrap();
    assert!((fk_x - 1.0).abs() < 1e-4);
    assert!((fk_y - 1.0).abs() < 1e-4);

    // 2. 雅可比行列式奇異點判定 (手肘伸直 theta2 = 0 奇異點 det = 0)
    let det_singular = robotics_motion_profile::robotics_planar_2link_jacobian_det(0.0, 1.0, 1.0);
    assert!(det_singular.abs() < 1e-6);
    let det_normal = robotics_motion_profile::robotics_planar_2link_jacobian_det(std::f64::consts::FRAC_PI_2, 1.0, 1.0);
    assert!((det_normal - 1.0).abs() < 1e-4);

    // 3. 梯形運動輪廓即時位移
    let d_acc = robotics_motion_profile::robotics_trapezoidal_profile_distance(1.0, 10.0, 10.0, 100.0).unwrap();
    assert!((d_acc - 5.0).abs() < 1e-3);
    let d_cruise = robotics_motion_profile::robotics_trapezoidal_profile_distance(2.0, 10.0, 10.0, 100.0).unwrap();
    assert!((d_cruise - 15.0).abs() < 1e-3);

    // 4. S曲線 第一階段躍度 (Jerk) 速度
    let v_jerk = robotics_motion_profile::robotics_scurve_jerk_velocity(0.5, 100.0).unwrap();
    assert!((v_jerk - 12.5).abs() < 1e-4);

    // 5. 步進馬達 Q15 微步正余弦驅動查表
    let (s0, c0) = robotics_motion_profile::robotics_stepper_microstep_sin_cos(0, 16).unwrap();
    assert_eq!(s0, 0);
    assert_eq!(c0, 32767);
    let (s16, c16) = robotics_motion_profile::robotics_stepper_microstep_sin_cos(16, 16).unwrap();
    assert_eq!(s16, 32767);
    assert_eq!(c16, 0);

    // 6. 關節軟極限夾止與警告區
    let (clamped, warn) = robotics_motion_profile::robotics_joint_soft_limit_clamp(3.14, 0.0, 3.0, 0.2);
    assert_eq!(clamped, 3.0);
    assert!(warn);

    // 7. 滾珠螺桿導程線性位移
    let dist = robotics_motion_profile::robotics_lead_screw_linear_distance(10.0, 5.0, 2.0).unwrap();
    assert_eq!(dist, 25.0);

    // 8. 機械臂負載力矩
    let torque = robotics_motion_profile::robotics_arm_payload_torque(1.0, 10.0, 9.8, 2.0).unwrap();
    assert_eq!(torque, 98.0 + 20.0);

    // 9. SCARA Z 軸進給率
    let feed = robotics_motion_profile::robotics_scara_z_axis_feed_rate(10.0, 600.0).unwrap();
    assert_eq!(feed, 100.0);
}

#[test]
fn test_ai_tensor_gemm_bricks() {
    use rust_bricks::ai_tensor_gemm;

    // 1. 分塊矩陣乘法 GEMM (2x2 * 2x2)
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [5.0f32, 6.0, 7.0, 8.0];
    let mut c = [0.0f32; 4];
    assert!(ai_tensor_gemm::gemm_tiled_f32_step(&a, &b, &mut c, 2, 2, 2, 1));
    // C00 = 1*5 + 2*7 = 19, C01 = 1*6 + 2*8 = 22
    // C10 = 3*5 + 4*7 = 43, C11 = 3*6 + 4*8 = 50
    assert_eq!(c, [19.0, 22.0, 43.0, 50.0]);

    // 2. Conv2D 卷積步進 (3x3 輸入, 2x2 卷積核 -> 2x2 輸出)
    let in_img = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let kernel = [1.0f32, 0.0, 0.0, 1.0]; // 對角卷積核
    let mut out_conv = [0.0f32; 4];
    assert!(ai_tensor_gemm::conv2d_valid_forward_step(&in_img, &kernel, &mut out_conv, 3, 3, 2, 2));
    assert_eq!(out_conv[0], 1.0 + 5.0); // 6.0
    assert_eq!(out_conv[1], 2.0 + 6.0); // 8.0
    assert_eq!(out_conv[2], 4.0 + 8.0); // 12.0
    assert_eq!(out_conv[3], 5.0 + 9.0); // 14.0

    // 3. MaxPool2D 2x2 池化
    let mut out_pool = [0.0f32; 1];
    let pool_in = [1.0f32, 9.0, 4.0, 2.0];
    assert!(ai_tensor_gemm::maxpool2d_2x2_stride2_step(&pool_in, &mut out_pool, 2, 2));
    assert_eq!(out_pool[0], 9.0);

    // 4. Batch Normalization 前向推論
    let x = [10.0f32, 20.0];
    let mean = [10.0f32];
    let var = [25.0f32]; // std = 5
    let gamma = [1.0f32];
    let beta = [0.0f32];
    let mut out_bn = [0.0f32; 2];
    assert!(ai_tensor_gemm::batch_norm_forward_inference(&x, &mean, &var, &gamma, &beta, 1e-5, &mut out_bn));
    assert!((out_bn[0] - 0.0).abs() < 1e-3);
    assert!((out_bn[1] - 2.0).abs() < 1e-3);

    // 5. Inverted Dropout 遮罩應用
    let mut x_drop = [10.0f32, 10.0];
    let mask_rand = [0u32, u32::MAX]; // 保留第一個，拋棄第二個
    assert!(ai_tensor_gemm::dropout_inverted_mask_apply(&mut x_drop, &mask_rand, 0.5));
    assert_eq!(x_drop[0], 20.0); // 縮放 1 / 0.5 = 2.0
    assert_eq!(x_drop[1], 0.0);

    // 6. Embedding 查找
    let emb_table = [0.1f32, 0.2, 0.3, 0.4, 0.5, 0.6];
    let mut emb_vec = [0.0f32; 3];
    assert!(ai_tensor_gemm::embedding_lookup_1d(&emb_table, 1, 3, &mut emb_vec));
    assert_eq!(emb_vec, [0.4, 0.5, 0.6]);

    // 7. LeakyReLU 與 Smooth L1 Loss
    assert!((ai_tensor_gemm::leaky_relu_f32(-10.0, 0.01) - (-0.1)).abs() < 1e-6);
    assert_eq!(ai_tensor_gemm::leaky_relu_f32(10.0, 0.01), 10.0);
    assert_eq!(ai_tensor_gemm::smooth_l1_loss_step(0.5, 0.0, 1.0), 0.125);

    // 8. 餘弦相似度
    let sim = ai_tensor_gemm::cosine_similarity_f32(&[1.0, 0.0], &[0.0, 1.0]).unwrap();
    assert_eq!(sim, 0.0);
    let sim_same = ai_tensor_gemm::cosine_similarity_f32(&[2.0, 3.0], &[4.0, 6.0]).unwrap();
    assert!((sim_same - 1.0).abs() < 1e-4);

    // 9. One-Hot 編碼
    let mut oh = [0u8; 4];
    assert!(ai_tensor_gemm::one_hot_encode_u8(2, 4, &mut oh));
    assert_eq!(oh, [0, 0, 1, 0]);
}
