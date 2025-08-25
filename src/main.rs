use rdev::{Event, EventType, Key, listen};
use rodio::OutputStream;
use rodio::buffer::SamplesBuffer;
use std::sync::Arc;

use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs, panic,
    sync::Mutex,
};

#[derive(Deserialize, Debug)]
struct SoundPack {
    definitions: HashMap<String, KeyDefinition>,
}
#[derive(Deserialize, Debug)]
struct KeyDefinition {
    timing: Vec<[f32; 2]>, // Array of [start_ms, end_ms] pairs
}

pub struct AudioSystem {
    stream: OutputStream, // mantiene vivo lo stream
}

impl AudioSystem {
    pub fn new() -> Self {
        // In rodio 0.21 si usa open_default_stream()
        let stream = rodio::OutputStreamBuilder::open_default_stream()
            .expect("Impossibile aprire lo stream audio predefinito");

        Self { stream: stream }
    }

    pub fn play_sound_segment(
        &self,
        key: &str,
        start: f32,
        end: f32,
        is_keydown: bool,
        samples: &[f32],
        channels: u16,
        sample_rate: u32,
    ) {
        //let pcm_opt; // = self.keyboard_samples.lock().unwrap().clone();
        //if let Some((samples, channels, sample_rate)) = pcm_opt

        // Calculate total audio duration in milliseconds
        let total_duration =
            ((samples.len() as f32) / (sample_rate as f32) / (channels as f32)) * 1000.0;

        // Calculate duration from start and end times
        let duration = end - start;

        // Validate input parameters
        if start < 0.0 || duration <= 0.0 || end <= start {
            eprintln!(
                "❌ Invalid time parameters for key '{}': start={:.3}ms, end={:.3}ms, duration={:.3}ms",
                key, start, end, duration
            );
            return;
        }
        // Use epsilon tolerance for floating point comparison (1ms tolerance)
        const EPSILON: f32 = 1.0; // 1ms tolerance
        println!(
            "🔍 Playing sound for key '{}': start={:.3}ms, end={:.3}ms, duration={:.3}ms (total duration: {:.3}ms) IsKeyDown:{}",
            key, start, end, duration, total_duration, is_keydown
        );

        // Check if start time exceeds audio duration - this is an error condition
        if start >= total_duration + EPSILON {
            eprintln!(
                "❌ TIMING ERROR: Start time {:.3}ms exceeds audio duration {:.3}ms for key '{}'",
                start, total_duration, key
            );
            return;
        }

        // Check if end time exceeds audio duration
        if end > total_duration + EPSILON {
            eprintln!(
                "❌ TIMING ERROR: Audio segment {:.3}ms-{:.3}ms exceeds duration {:.3}ms for key '{}'",
                start, end, total_duration, key
            );
            return;
        }

        // Calculate sample positions (convert milliseconds to seconds for sample calculation)
        let start_sample = ((start / 1000.0) * (sample_rate as f32) * (channels as f32)) as usize;
        let end_sample = ((end / 1000.0) * (sample_rate as f32) * (channels as f32)) as usize;
        /*
        // Validate sample range with safety checks
        if end_sample > samples.len() {
                    // Try to clamp end_sample to available samples
                    let max_available_sample = samples.len();
                    let clamped_end_sample = max_available_sample;
                    let clamped_end_time =
                        ((clamped_end_sample as f32) / (sample_rate as f32) / (channels as f32)) * 1000.0;
                    let clamped_duration = clamped_end_time - start;

                    // Use clamped values if they're reasonable
                    if clamped_duration > 1.0 && clamped_end_sample > start_sample {
                        let segment_samples = samples[start_sample..clamped_end_sample].to_vec();
                        let segment = SamplesBuffer::new(channels, sample_rate, segment_samples);


                        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                            sink.set_volume(2.0);
                            sink.append(segment);

                            let mut key_sinks = self.key_sinks.lock().unwrap();
                            self.manage_active_sinks(&mut key_sinks);
                            key_sinks.insert(
                                format!("{}-{}", key, if is_keydown { "down" } else { "up" }),
                                sink,
                            );
                        }

                        return;
                    }

                    return;
                }
        */
        // Final validation before extracting samples
        if start_sample >= end_sample || start_sample >= samples.len() {
            eprintln!(
                "❌ INTERNAL ERROR: Invalid sample range for key '{}': {}..{} (max {})",
                key,
                start_sample,
                end_sample,
                samples.len()
            );
            eprintln!(
                "   Audio: {:.3}ms, Channels: {}, Rate: {}",
                total_duration, channels, sample_rate
            );
            return;
        }
        //println!("inizio suono:{:?}", samples);

        /*let freq_left = 440.0;
        let freq_right = 880.0;
        let channels = 2;
        let sample_rate = 44100;
        let duration_secs = 0.5;

        let total_frames = (sample_rate as f32 * duration_secs) as usize;

        let mut segment_samples = Vec::with_capacity(total_frames * channels as usize);
        for n in 0..total_frames {
            let t = n as f32 / sample_rate as f32;
            // Esempi sinusoidi
            let l = (2.0 * std::f32::consts::PI * freq_left * t).sin();
            let r = (2.0 * std::f32::consts::PI * freq_right * t).sin();
            segment_samples.push(l);
            segment_samples.push(r);
        }*/

        let segment_samples = samples[start_sample..end_sample].to_vec();

        // println!("segment_samples:{:?}", segment_samples);
        let segment = SamplesBuffer::new(channels, sample_rate, segment_samples);
        // let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        // .expect("Impossibile aprire lo stream audio predefinito");
        let sink = rodio::Sink::connect_new(&self.stream.mixer());
        sink.set_volume(1.0);
        sink.append(segment);

        sink.detach();

        //sink.sleep_until_end();
        // eprintln!("fine suono");
        /*
        if let Ok(sink) = Sink::try_new(&stream_handle) {
            sink.set_volume(1.0f32);
            sink.append(segment);


            let mut key_sinks = self.key_sinks.lock().unwrap();
            self.manage_active_sinks(&mut key_sinks);
            key_sinks.insert(
                format!("{}-{}", key, if is_keydown { "down" } else { "up" }),
                sink
            );

        }

        */
    }
}

fn main() {
    let sound_file_path = "./soundtrack/eg-crystal-purple/sound.ogg";
    let config_content = "./soundtrack/eg-crystal-purple/config.json";

    //********
    println!("📂 Loading Json config");

    // Leggi il file JSON
    let config_content = match fs::read_to_string(config_content) {
        Ok(content) => content,
        Err(e) => {
            //println!("Errore nella lettura del file: {}", e);
            panic!("Errore nella lettura del file: {}", e);
        }
    };
    println!("📂 Loading keyboard soundpack ");

    // Parsing del JSON
    let sound_pack: SoundPack = match serde_json::from_str(&config_content) {
        Ok(pack) => pack,
        Err(e) => {
            //println!("Errore nel parsing JSON: {}", e);
            panic!("Errore nel parsing JSON: {}", e);
        }
    };

    // Mostra alcune informazioni sul soundpack
    println!(
        "🔊 Soundpack loaded with {} keys defined",
        sound_pack.definitions.len()
    );

    // *******
    // print!("samples:{:?}", samples);
    println!("📂 creating keyboard mappings");

    let key_mappings = create_key_mappings(&sound_pack); // Update audio context with keyboard data

    // print!("key_mappings:{:?}", key_mappings);

    let (samples, channels, sample_rate) = match load_audio_with_symphonia(&sound_file_path) {
        Ok((samples, channels, sample_rate)) => (samples, channels, sample_rate),
        Err(e) => {
            panic!("❌ Failed to load audio: {}", e);
        }
    };

    // Ora puoi usare samples, channels e sample_rate più volte
    // play_sound_segment("a", 2000.0, 2500.0, true, &samples, channels, sample_rate);
    // play_sound_segment("b", 1000.0, 1200.0, true, &samples, channels, sample_rate);
    // ecc...

    let pressed_keys = Arc::new(Mutex::new(HashSet::new()));
    let audio_system = Arc::new(AudioSystem::new());

    let pressed_keys_clone = pressed_keys.clone();
    if let Err(error) = listen(move |event| {
        callback(
            event,
            &pressed_keys_clone,
            &key_mappings,
            &samples,
            channels,
            sample_rate,
            &audio_system,
        );
    }) {
        println!("Errore: {:?}", error);
    }
}

fn load_audio_with_symphonia(file_path: &str) -> Result<(Vec<f32>, u16, u32), String> {
    use std::fs::File;
    use symphonia::core::audio::{AudioBufferRef, Signal};
    use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    // First, check if file exists and has content
    let metadata =
        std::fs::metadata(file_path).map_err(|e| format!("Failed to get file metadata: {}", e))?;
    if metadata.len() == 0 {
        return Err(format!("Audio file is empty: {}", file_path));
    }

    let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = std::path::Path::new(file_path).extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| {
            format!(
                "Failed to probe format for '{}': {} (file size: {} bytes)",
                file_path,
                e,
                metadata.len()
            )
        })?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks found")?;

    let dec_opts: DecoderOptions = Default::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .map_err(|e| format!("Failed to create decoder: {}", e))?;

    let track_id = track.id;
    let mut samples = Vec::new();
    let mut sample_rate = 44100u32;
    let mut channels = 2u16;

    // Decode audio packets
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => {
                break;
            } // End of stream
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                if samples.is_empty() {
                    // Get format info from first decoded buffer
                    sample_rate = decoded.spec().rate;
                    channels = decoded.spec().channels.count() as u16;
                } // Convert audio buffer to f32 samples
                match decoded {
                    AudioBufferRef::F32(buf) => {
                        if channels == 1 {
                            // Mono audio
                            for &sample in buf.chan(0) {
                                samples.push(sample);
                            }
                        } else {
                            // Stereo audio - interleave samples correctly
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push(*left);
                                samples.push(*right);
                            }
                        }
                    }
                    AudioBufferRef::S32(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push((sample as f32) / (i32::MAX as f32));
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push((*left as f32) / (i32::MAX as f32));
                                samples.push((*right as f32) / (i32::MAX as f32));
                            }
                        }
                    }
                    AudioBufferRef::S16(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push((sample as f32) / (i16::MAX as f32));
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push((*left as f32) / (i16::MAX as f32));
                                samples.push((*right as f32) / (i16::MAX as f32));
                            }
                        }
                    }
                    AudioBufferRef::U32(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push(
                                    ((sample as f32) - (u32::MAX as f32) / 2.0)
                                        / ((u32::MAX as f32) / 2.0),
                                );
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push(
                                    ((*left as f32) - (u32::MAX as f32) / 2.0)
                                        / ((u32::MAX as f32) / 2.0),
                                );
                                samples.push(
                                    ((*right as f32) - (u32::MAX as f32) / 2.0)
                                        / ((u32::MAX as f32) / 2.0),
                                );
                            }
                        }
                    }
                    AudioBufferRef::U16(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push(
                                    ((sample as f32) - (u16::MAX as f32) / 2.0)
                                        / ((u16::MAX as f32) / 2.0),
                                );
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push(
                                    ((*left as f32) - (u16::MAX as f32) / 2.0)
                                        / ((u16::MAX as f32) / 2.0),
                                );
                                samples.push(
                                    ((*right as f32) - (u16::MAX as f32) / 2.0)
                                        / ((u16::MAX as f32) / 2.0),
                                );
                            }
                        }
                    }
                    AudioBufferRef::U8(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push(((sample as f32) - 128.0) / 128.0);
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push(((*left as f32) - 128.0) / 128.0);
                                samples.push(((*right as f32) - 128.0) / 128.0);
                            }
                        }
                    }
                    AudioBufferRef::S8(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push((sample as f32) / (i8::MAX as f32));
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push((*left as f32) / (i8::MAX as f32));
                                samples.push((*right as f32) / (i8::MAX as f32));
                            }
                        }
                    }
                    AudioBufferRef::F64(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push(sample as f32);
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push(*left as f32);
                                samples.push(*right as f32);
                            }
                        }
                    }
                    AudioBufferRef::U24(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                let sample_f32 = ((sample.inner() as f32) - 8388608.0) / 8388608.0; // 2^23
                                samples.push(sample_f32);
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                let left_f32 = ((left.inner() as f32) - 8388608.0) / 8388608.0;
                                let right_f32 = ((right.inner() as f32) - 8388608.0) / 8388608.0;
                                samples.push(left_f32);
                                samples.push(right_f32);
                            }
                        }
                    }
                    AudioBufferRef::S24(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                let sample_f32 = (sample.inner() as f32) / 8388607.0; // 2^23 - 1
                                samples.push(sample_f32);
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                let left_f32 = (left.inner() as f32) / 8388607.0;
                                let right_f32 = (right.inner() as f32) / 8388607.0;
                                samples.push(left_f32);
                                samples.push(right_f32);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("⚠️ [DEBUG] Decode error (continuing): {}", e);
                continue;
            }
        }
    }

    if samples.is_empty() {
        return Err("No audio data decoded".to_string());
    }

    Ok((samples, channels, sample_rate))
}
fn create_key_mappings(
    soundpack: &SoundPack,
) -> std::collections::HashMap<String, Vec<(f64, f64)>> {
    let mut key_mappings = std::collections::HashMap::new(); // For keyboard soundpacks, use the definitions field for keyboard mappings
    // println!("definitions:{:?}", soundpack.definitions);
    for (key, key_def) in &soundpack.definitions {
        // Convert KeyDefinition timing to Vec<(f64, f64)>
        let converted_mappings: Vec<(f64, f64)> = key_def
            .timing
            .iter()
            .map(|pair| (pair[0] as f64, pair[1] as f64))
            .collect();
        key_mappings.insert(key.clone(), converted_mappings);
    }

    key_mappings
}

fn callback(
    event: Event,
    pressed_keys: &Arc<Mutex<HashSet<Key>>>,
    key_mappings: &HashMap<String, Vec<(f64, f64)>>,
    samples: &Vec<f32>,
    channels: u16,
    sample_rate: u32,
    audio_system: &AudioSystem,
) {
    match event.event_type {
        EventType::KeyPress(key) => {
            let mut keys = pressed_keys.lock().unwrap();
            if keys.insert(key) {
                // prima volta che questo tasto viene premuto
                if let Some(name) = &event.name {
                    println!("Tasto premuto: {}", name);
                }
                let key_str = format!("{:?}", key);
                if let Some(vec_ref) = key_mappings.get(&key_str) {
                    // `vec_ref` è un riferimento: &Vec<(f64, f64)>
                    //  println!("Trovato: {:?}", vec_ref[0].0);
                    audio_system.play_sound_segment(
                        &key_str,
                        vec_ref[0].0 as f32,
                        vec_ref[0].1 as f32,
                        true,
                        &samples,
                        channels,
                        sample_rate,
                    );
                } else {
                    println!("Chiave non trovata: {}", key_str);
                }
                //println!("Samples:{:?}", samples);
            }
        }
        EventType::KeyRelease(key) => {
            let mut keys = pressed_keys.lock().unwrap();
            if keys.remove(&key) {
                if let Some(name) = &event.name {
                    println!("Tasto rilasciato: {}", name);
                }
                let key_str = format!("{:?}", key);
                if let Some(vec_ref) = key_mappings.get(&key_str) {
                    // `vec_ref` è un riferimento: &Vec<(f64, f64)>
                    // println!("Trovato: {:?}", vec_ref);
                    audio_system.play_sound_segment(
                        &key_str,
                        vec_ref[1].0 as f32,
                        vec_ref[1].1 as f32,
                        false,
                        &samples,
                        channels,
                        sample_rate,
                    );
                } else {
                    println!("Chiave non trovata: {}", key_str);
                }
            }
        }
        _ => {}
    }
}
