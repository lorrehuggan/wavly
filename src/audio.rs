use anyhow::{Context, Result};
use std::{fs::File, io::ErrorKind, path::Path, time::Duration};
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, errors::Error, formats::FormatOptions,
    io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
};

#[derive(Debug)]
pub struct DecodedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub duration: Duration,
}

pub fn decode_audio_file(path: &Path) -> Result<DecodedAudio> {
    let file =
        Box::new(File::open(path).with_context(|| format!("failed to open {}", path.display()))?);
    let mss = MediaSourceStream::new(file, Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        hint.with_extension(ext);
    }

    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .with_context(|| format!("failed to probe audio format for {}", path.display()))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|track| {
            track.codec_params.sample_rate.is_some() && track.codec_params.channels.is_some()
        })
        .context("no audio track found")?;

    let codec_params = track.codec_params.clone();
    let track_id = track.id;
    let sample_rate = codec_params
        .sample_rate
        .context("audio track is missing a sample rate")?;
    let channel_count = codec_params
        .channels
        .context("audio track is missing channel information")?
        .count();

    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &DecoderOptions::default())
        .context("failed to create decoder")?;

    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let mut interleaved = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::IoError(err)) if err.kind() == ErrorKind::UnexpectedEof => break,
            Err(Error::IoError(err)) => {
                return Err(anyhow::anyhow!(
                    "failed to read packet from {}: {}",
                    path.display(),
                    err
                ));
            }
            Err(Error::DecodeError(err)) => {
                return Err(anyhow::anyhow!(
                    "failed to decode packet from {}: {}",
                    path.display(),
                    err
                ));
            }
            Err(Error::ResetRequired) => continue,
            Err(err) => {
                return Err(anyhow::anyhow!(
                    "failed to process {}: {}",
                    path.display(),
                    err
                ));
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                if sample_buf.is_none() {
                    let spec = *audio_buf.spec();
                    let duration = audio_buf.capacity() as u64;
                    sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                }

                if let Some(buf) = sample_buf.as_mut() {
                    buf.copy_interleaved_ref(audio_buf);
                    interleaved.extend_from_slice(buf.samples());
                }
            }
            Err(Error::DecodeError(_)) => continue,
            Err(Error::IoError(err)) if err.kind() == ErrorKind::UnexpectedEof => break,
            Err(Error::ResetRequired) => continue,
            Err(err) => {
                return Err(anyhow::anyhow!(
                    "failed to decode {}: {}",
                    path.display(),
                    err
                ));
            }
        }
    }

    let mono_samples = to_mono(&interleaved, channel_count);
    let duration = if let Some(frames) = codec_params.n_frames {
        Duration::from_secs_f64(frames as f64 / sample_rate as f64)
    } else {
        Duration::from_secs_f64(mono_samples.len() as f64 / sample_rate as f64)
    };

    Ok(DecodedAudio {
        samples: mono_samples,
        sample_rate,
        duration,
    })
}

fn to_mono(samples: &[f32], channel_count: usize) -> Vec<f32> {
    if channel_count <= 1 {
        return samples.to_vec();
    }

    samples
        .chunks_exact(channel_count)
        .map(|frame| frame.iter().copied().sum::<f32>() / channel_count as f32)
        .collect()
}
