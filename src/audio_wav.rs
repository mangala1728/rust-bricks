// ## Map
// Role: core/audio_wav — 零外部依賴之純記憶體 16-bit PCM 單聲道 WAV 格式編解碼微積木。
// Inputs: PCM16 取樣切片、取樣率、或者 WAV 二進位位元組切片。
// Outputs: WAV 二進位位元組向量、或者解析後的 (PCM16 樣本, 取樣率)。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、長度防溢出守衛（最大 32,000,000 樣本）、控制流巢狀深度 <= 2。

const MAX_WAV_SAMPLES: usize = 32_000_000;
const MAX_WAV_BYTES: usize = 64_000_044;

/// @outputs: 1
/// @domain: data_len [0..4294967200]
/// @domain: sample_rate [8000..192000]
fn write_wav_headers(buf: &mut Vec<u8>, data_len: u32, sample_rate: u32) {
    if data_len > 4_294_967_200 || sample_rate < 8_000 || sample_rate > 192_000 {
        return;
    }
    let file_len = 36u32.saturating_add(data_len);
    let byte_rate = sample_rate.saturating_mul(2);
    let block_align: u16 = 2;
    let bits_per_sample: u16 = 16;
    let num_channels: u16 = 1;

    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_len.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&num_channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&bits_per_sample.to_le_bytes());

    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());
}

/// @outputs: 2
/// @domain: sample_rate [8000..192000]
pub fn encode_pcm16_wav(samples: &[i16], sample_rate: u32) -> Result<Vec<u8>, &'static str> {
    if sample_rate < 8_000 || sample_rate > 192_000 {
        return Err("取樣率超出常規音訊定義域 [8000..192000]");
    }
    if samples.len() > MAX_WAV_SAMPLES {
        return Err("音訊樣本數超過上限 (32,000,000)");
    }
    let data_len = match (samples.len() as u64).checked_mul(2) {
        Some(l) if l <= u32::MAX as u64 => l as u32,
        _ => return Err("音訊樣本容量溢出"),
    };

    let total_bytes = 44usize.saturating_add(samples.len() * 2);
    let mut out = Vec::with_capacity(total_bytes);

    write_wav_headers(&mut out, data_len, sample_rate);

    for &sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }

    Ok(out)
}

/// @outputs: 2
fn validate_wav_header(header: &[u8]) -> Result<(u32, usize), &'static str> {
    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
        return Err("無效的 RIFF WAVE 檔頭");
    }
    if &header[12..16] != b"fmt " {
        return Err("缺少 fmt 子區塊");
    }
    let audio_fmt = u16::from_le_bytes([header[20], header[21]]);
    if audio_fmt != 1 {
        return Err("僅支援 PCM 未壓縮格式 (Format 1)");
    }
    let num_channels = u16::from_le_bytes([header[22], header[23]]);
    if num_channels != 1 {
        return Err("僅支援單聲道 (Mono)");
    }
    let sample_rate = u32::from_le_bytes([header[24], header[25], header[26], header[27]]);
    if sample_rate < 8_000 || sample_rate > 192_000 {
        return Err("取樣率超出有效範圍");
    }
    if &header[36..40] != b"data" {
        return Err("缺少 data 子區塊");
    }
    let data_bytes = u32::from_le_bytes([header[40], header[41], header[42], header[43]]) as usize;
    Ok((sample_rate, data_bytes))
}

/// @outputs: 2
pub fn decode_pcm16_wav(wav_bytes: &[u8]) -> Result<(Vec<i16>, u32), &'static str> {
    if wav_bytes.len() < 44 {
        return Err("WAV 檔案長度小於標頭最低需求 (44 位元組)");
    }
    if wav_bytes.len() > MAX_WAV_BYTES {
        return Err("WAV 檔案長度超過上限");
    }

    let (sample_rate, data_bytes) = validate_wav_header(&wav_bytes[0..44])?;

    if wav_bytes.len() < 44 + data_bytes {
        return Err("WAV 實際資料位元組小於標頭宣告長度");
    }

    let num_samples = data_bytes / 2;
    if num_samples > MAX_WAV_SAMPLES {
        return Err("解析出的樣本數超過最大有界上限");
    }

    let mut samples = Vec::with_capacity(num_samples);
    let raw_payload = &wav_bytes[44..44 + data_bytes];

    for chunk in raw_payload.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        samples.push(sample);
    }

    Ok((samples, sample_rate))
}