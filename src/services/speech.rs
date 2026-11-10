use crate::domain::models::{HistoryEntry, OutputFormat, VoiceInfo};
use crate::services::mp3_encoder::wav_to_mp3;
use chrono::Local;
use sapi_lite::audio::{AudioFormat, AudioStream, BitRate, Channels, SampleRate};
use sapi_lite::tts::{installed_voices, Rate, SpeechOutput, SyncSynthesizer, Voice, Volume};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const WAV_SAMPLE_RATE: SampleRate = SampleRate::Hz44100;

fn with_sapi<T>(operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    sapi_lite::initialize()
        .map_err(|error| format!("Não foi possível inicializar a SAPI do Windows: {error}"))?;

    let result = operation();

    // Em sapi_lite 0.1.1, finalize() retorna unit (()), não Result.
    // Portanto, apenas encerramos a inicialização COM/SAPI e preservamos
    // o resultado da operação executada acima.
    sapi_lite::finalize();

    result
}

fn collect_installed_voices() -> Result<Vec<Voice>, String> {
    let iterator = installed_voices(None, None)
        .map_err(|error| format!("Não foi possível consultar as vozes instaladas: {error}"))?;
    Ok(iterator.collect())
}

pub fn list_voices() -> Result<Vec<VoiceInfo>, String> {
    with_sapi(|| {
        let voices = collect_installed_voices()?;
        let result = voices
            .into_iter()
            .enumerate()
            .map(|(index, voice)| VoiceInfo {
                index,
                name: voice
                    .name()
                    .map(|name| name.to_string_lossy().to_string())
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or_else(|| format!("Voz {}", index + 1)),
                language: voice
                    .language()
                    .map(|language| language.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Idioma não informado".to_string()),
            })
            .collect();
        Ok(result)
    })
}

fn selected_voice(index: usize) -> Result<Voice, String> {
    collect_installed_voices()?
        .into_iter()
        .nth(index)
        .ok_or_else(|| "A voz selecionada não está mais instalada no Windows.".to_string())
}

fn configure_synthesizer(
    synthesizer: &SyncSynthesizer,
    voice_index: usize,
    rate: i32,
    volume: u32,
) -> Result<(), String> {
    let voice = selected_voice(voice_index)?;
    synthesizer
        .set_voice(&voice)
        .map_err(|error| format!("Não foi possível selecionar a voz: {error}"))?;
    synthesizer
        .set_rate(Rate::new(rate.clamp(-10, 10)))
        .map_err(|error| format!("Não foi possível ajustar a velocidade: {error}"))?;
    synthesizer
        .set_volume(Volume::new(volume.clamp(0, 100)))
        .map_err(|error| format!("Não foi possível ajustar o volume: {error}"))?;
    Ok(())
}

pub fn test_voice(voice_index: usize, rate: i32, volume: u32) -> Result<(), String> {
    with_sapi(|| {
        let synthesizer = SyncSynthesizer::new()
            .map_err(|error| format!("Não foi possível iniciar o sintetizador: {error}"))?;
        configure_synthesizer(&synthesizer, voice_index, rate, volume)?;
        synthesizer
            .speak(
                "Olá! Esta é uma prévia de voz do Rust Voice Studio.",
                Some(Duration::from_secs(30)),
            )
            .map_err(|error| format!("Falha ao reproduzir a prévia: {error}"))
    })
}

/// Sintetiza e reproduz o texto informado no dispositivo de áudio padrão do Windows.
/// A função é chamada em uma thread de trabalho para não bloquear a interface gráfica.
pub fn speak_text(
    text: &str,
    voice_index: usize,
    rate: i32,
    volume: u32,
) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("Digite algum texto antes de iniciar a leitura.".to_string());
    }

    with_sapi(|| {
        let synthesizer = SyncSynthesizer::new()
            .map_err(|error| format!("Não foi possível iniciar o sintetizador: {error}"))?;
        configure_synthesizer(&synthesizer, voice_index, rate, volume)?;
        synthesizer
            .speak(text, None)
            .map_err(|error| format!("Falha ao reproduzir o texto: {error}"))
    })
}

fn render_wav(
    destination: &Path,
    text: &str,
    voice_index: usize,
    rate: i32,
    volume: u32,
) -> Result<(), String> {
    let format = AudioFormat {
        sample_rate: WAV_SAMPLE_RATE,
        bit_rate: BitRate::Bits16,
        channels: Channels::Mono,
    };

    let stream = AudioStream::create_file(destination, &format)
        .map_err(|error| format!("Não foi possível criar o arquivo WAV: {error}"))?;
    let synthesizer = SyncSynthesizer::new()
        .map_err(|error| format!("Não foi possível iniciar o sintetizador: {error}"))?;

    configure_synthesizer(&synthesizer, voice_index, rate, volume)?;
    synthesizer
        .set_output(SpeechOutput::Stream(stream), false)
        .map_err(|error| format!("Não foi possível direcionar a fala para o arquivo: {error}"))?;
    synthesizer
        .speak(text, None)
        .map_err(|error| format!("Falha durante a síntese de voz: {error}"))?;

    // O sintetizador é descartado antes da leitura/conversão do arquivo para liberar o fluxo.
    drop(synthesizer);
    Ok(())
}

fn temporary_wav_path() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("rust_voice_studio_{}_{}.wav", std::process::id(), nonce))
}

pub fn export_audio(
    text: String,
    voice_index: usize,
    rate: i32,
    volume: u32,
    destination: PathBuf,
    output_format: OutputFormat,
    bitrate_kbps: u32,
) -> Result<HistoryEntry, String> {
    if text.trim().is_empty() {
        return Err("Digite algum texto antes de gerar o áudio.".to_string());
    }

    let destination = destination.with_extension(output_format.extension());
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Não foi possível criar a pasta de destino: {error}"))?;
    }

    match output_format {
        OutputFormat::Wav => {
            let result = with_sapi(|| render_wav(&destination, &text, voice_index, rate, volume));
            if let Err(error) = result {
                let _ = std::fs::remove_file(&destination);
                return Err(error);
            }
        }
        OutputFormat::Mp3 => {
            let temporary_wav = temporary_wav_path();
            let render_result = with_sapi(|| {
                render_wav(&temporary_wav, &text, voice_index, rate, volume)
            });

            if let Err(error) = render_result {
                let _ = std::fs::remove_file(&temporary_wav);
                return Err(error);
            }

            let conversion_result = wav_to_mp3(&temporary_wav, &destination, bitrate_kbps);
            let _ = std::fs::remove_file(&temporary_wav);
            if let Err(error) = conversion_result {
                let _ = std::fs::remove_file(&destination);
                return Err(error);
            }
        }
    }

    let title = destination
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "audio-gerado".to_string());

    Ok(HistoryEntry {
        title,
        path: destination,
        format: output_format.label().to_string(),
        characters: text.chars().count(),
        created_at: Local::now().format("%d/%m/%Y %H:%M:%S").to_string(),
    })
}
