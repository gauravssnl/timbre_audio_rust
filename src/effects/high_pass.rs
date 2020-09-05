use crate::{
    core::{AudioBuffer, AudioSource, SharedAudioSource},
    ReadResult,
};

use tracing::trace_span;

pub struct HighPass {
    buffer: Vec<f32>,
    rc: f32,
    source: SharedAudioSource,
    prev: [f32; 2],
}

impl HighPass {
    pub fn new(source: SharedAudioSource, cutoff: f32) -> Self {
        let buffer = Vec::new();
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        HighPass {
            buffer,
            rc,
            source,
            prev: [0.0, 0.0],
        }
    }
}

impl AudioSource for HighPass {
    fn read(&mut self, buffer: &mut AudioBuffer) -> ReadResult {
        let span = trace_span!("HighPass::read");
        let _span = span.enter();

        let result = self.source.lock().unwrap().read(buffer);
        let written = result.read;
        if written == 0 {
            return result;
        }
        self.buffer.resize(buffer.samples.len(), 0.0);

        match buffer.format.channels {
            1 => {
                let dt = 1.0 / buffer.format.sample_rate as f32;
                self.prev = filter_mono(
                    &mut buffer.samples[..written],
                    &mut self.buffer[..written],
                    dt,
                    self.rc,
                    self.prev,
                );
            }
            2 => {
                let dt = 1.0 / buffer.format.sample_rate as f32;
                self.prev = filter_stereo(
                    &mut buffer.samples[..written],
                    &mut self.buffer[..written],
                    dt,
                    self.rc,
                    self.prev,
                );
            }
            _ => panic!("Unsupported channel count."),
        }

        result
    }
}

fn filter_mono(
    samples: &mut [f32],
    buffer: &mut [f32],
    dt: f32,
    rc: f32,
    prev: [f32; 2],
) -> [f32; 2] {
    assert!(!samples.is_empty() && !buffer.is_empty());
    assert!(buffer.len() >= samples.len());

    let a = rc / (rc + dt);
    let res = [samples[samples.len() - 1], 0.0];

    buffer[0] = a * (buffer[buffer.len() - 1] + samples[0] - prev[0]);
    for i in 1..buffer.len() {
        buffer[i] = a * (buffer[i - 1] + samples[i] - samples[i - 1]);
    }
    samples.copy_from_slice(&buffer[..samples.len()]);

    res
}

fn filter_stereo(
    samples: &mut [f32],
    buffer: &mut [f32],
    dt: f32,
    rc: f32,
    prev: [f32; 2],
) -> [f32; 2] {
    assert!(!samples.is_empty() && !buffer.is_empty());
    assert!(samples.len() % 2 == 0 && buffer.len() % 2 == 0);
    assert!(buffer.len() >= samples.len());

    let a = rc / (rc + dt);
    let res = [samples[samples.len() - 2], samples[samples.len() - 1]];

    buffer[0] = a * (buffer[buffer.len() - 2] + samples[0] - prev[0]);
    buffer[1] = a * (buffer[buffer.len() - 1] + samples[1] - prev[1]);
    for i in 2..buffer.len() {
        buffer[i] = a * (buffer[i - 2] + samples[i] - samples[i - 2]);
    }
    samples.copy_from_slice(&buffer[..samples.len()]);

    res
}
