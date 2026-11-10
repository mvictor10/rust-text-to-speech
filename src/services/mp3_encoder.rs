use std::path::Path;

/// Converte o WAV PCM de 16 bits produzido pela SAPI em MP3 localmente.
pub fn wav_to_mp3(wav_path: &Path, mp3_path: &Path, bitrate_kbps: u32) -> Result<(), String> {
    let mut reader = hound::WavReader::open(wav_path)
        .map_err(|error| format!("Não foi possível abrir o WAV intermediário: {error}"))?;
    let spec = reader.spec();

    if spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample != 16 {
        return Err(format!(
            "Formato WAV não suportado pelo conversor: {:?}, {} bits.",
            spec.sample_format, spec.bits_per_sample
        ));
    }

    if !(spec.channels == 1 || spec.channels == 2) {
        return Err(format!("Quantidade de canais não suportada: {}.", spec.channels));
    }

    let mut pcm = Vec::new();
    for sample in reader.samples::<i16>() {
        let value = sample.map_err(|error| format!("Falha ao ler amostra WAV: {error}"))?;
        pcm.push(value as f32 / 32768.0);
    }

    if pcm.is_empty() {
        return Err("A síntese não produziu amostras de áudio.".to_string());
    }

    let bitrate = bitrate_kbps.clamp(64, 320);
    let mut encoder = rusty_mp3::Mp3Encoder::new(rusty_mp3::Mp3EncoderConfig {
        bitrate_kbps: bitrate,
        vbr_quality: None,
    });

    encoder
        .push_pcm_f32(&pcm, spec.channels, spec.sample_rate)
        .map_err(|error| format!("Falha ao preparar o codificador MP3: {error}"))?;
    encoder.finish();

    let mut encoded = Vec::new();
    while let Ok(packet) = encoder.next_packet() {
        encoded.extend_from_slice(&packet);
    }

    if encoded.is_empty() {
        return Err("O codificador não retornou dados MP3.".to_string());
    }

    std::fs::write(mp3_path, encoded)
        .map_err(|error| format!("Não foi possível salvar o MP3: {error}"))
}
