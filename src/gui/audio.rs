use anyhow::Result;
use log::{error, info, warn};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    bgm_sink: Option<Sink>,
    se_sinks: Vec<Sink>,
    pcm_sinks: HashMap<i32, Sink>,
    base_dir: PathBuf,
}

impl AudioManager {
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        Ok(Self {
            _stream,
            stream_handle,
            bgm_sink: None,
            se_sinks: Vec::new(),
            pcm_sinks: HashMap::new(),
            base_dir,
        })
    }

    fn find_file(&self, name: &str) -> Option<PathBuf> {
        for ext in ["ogg", "wav", "mp3", "flac"] {
            for dir in ["", "BGM", "SE", "KOE"] {
                let mut p = if dir.is_empty() {
                    self.base_dir.clone()
                } else {
                    self.base_dir.join(dir)
                };
                p.push(format!("{}.{}", name, ext));
                if p.exists() {
                    return Some(p);
                }

                // try lowercase if exact doesn't match
                let lower_name = name.to_lowercase();
                let p_lower = if dir.is_empty() {
                    self.base_dir.clone()
                } else {
                    self.base_dir.join(dir)
                }
                .join(format!("{}.{}", lower_name, ext));
                if p_lower.exists() {
                    return Some(p_lower);
                }
            }
        }
        None
    }

    pub fn play_bgm(&mut self, name: &str, loop_flag: bool, fade_in_ms: i32) {
        if let Some(path) = self.find_file(name) {
            match File::open(&path) {
                Ok(file) => {
                    if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
                        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                            if loop_flag {
                                sink.append(decoder.repeat_infinite());
                            } else {
                                sink.append(decoder);
                            }
                            sink.set_volume(if fade_in_ms > 0 { 0.5 } else { 1.0 });
                            sink.play();
                            self.bgm_sink = Some(sink);
                            info!("Playing BGM: {}", name);
                        }
                    } else {
                        error!("Failed to decode BGM: {}", path.display());
                    }
                }
                Err(e) => error!("Failed to open BGM {}: {}", path.display(), e),
            }
        } else {
            warn!("BGM file not found: {}", name);
        }
    }

    pub fn stop_bgm(&mut self, _fade_out_ms: i32) {
        if let Some(sink) = self.bgm_sink.take() {
            sink.stop();
            info!("Stopped BGM");
        }
    }

    pub fn play_se(&mut self, name: &str) {
        if let Some(path) = self.find_file(name) {
            match File::open(&path) {
                Ok(file) => {
                    if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
                        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                            sink.append(decoder);
                            sink.play();
                            self.se_sinks.retain(|s| !s.empty());
                            self.se_sinks.push(sink);
                            info!("Playing SE: {}", name);
                        }
                    } else {
                        error!("Failed to decode SE: {}", path.display());
                    }
                }
                Err(e) => error!("Failed to open SE {}: {}", path.display(), e),
            }
        } else {
            warn!("SE file not found: {}", name);
        }
    }

    pub fn stop_se(&mut self) {
        for sink in self.se_sinks.drain(..) {
            sink.stop();
        }
        info!("Stopped all SE");
    }

    pub fn play_pcmch(&mut self, ch: i32, name: &str, loop_flag: bool) {
        if let Some(path) = self.find_file(name) {
            match File::open(&path) {
                Ok(file) => {
                    if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
                        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                            if loop_flag {
                                sink.append(decoder.repeat_infinite());
                            } else {
                                sink.append(decoder);
                            }
                            sink.play();
                            self.pcm_sinks.insert(ch, sink);
                            info!("Playing PCM CH{}: {}", ch, name);
                        }
                    } else {
                        error!("Failed to decode PCM: {}", path.display());
                    }
                }
                Err(e) => error!("Failed to open PCM {}: {}", path.display(), e),
            }
        } else {
            warn!("PCM file not found: {}", name);
        }
    }

    pub fn stop_pcmch(&mut self, ch: i32) {
        if let Some(sink) = self.pcm_sinks.remove(&ch) {
            sink.stop();
            info!("Stopped PCM CH{}", ch);
        }
    }
}
