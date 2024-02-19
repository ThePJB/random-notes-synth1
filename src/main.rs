
extern crate anyhow;
extern crate clap;
extern crate cpal;

mod kmath;
use kmath::*;

use rand_xoshiro::{Xoshiro256StarStar, rand_core::SeedableRng};
use rand_xoshiro::rand_core::RngCore;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;

// tensionizer
// tension could be as simple as time since returning, and effects the probability of going toward the root (initially away but over time closer)
// probably need an epsilon

// music player and generate simple patterns that will no doubt turn out to be amazing compositions
// architecture: probably it can play N chunks at a time, coordinate music from other thread

// rhythm too, chance to do it double or not, just like gball

// get actual harmony

// what if you only go up by an octave and down by a fifth

// yea its kinda cool to flip between some ratios
// it would also be fun to have 2 tones just moving around or something
// do shepherd tones
// harmony enumerator, is there a sensible order
// pythagorean intervals etc?
// maybe its how close to halfway but quantized

// interdimensional radio
// it would be nice to draw the spectrogram

// motivation for pythagorean intervals: you can construct everything out of octaves and fifths, hence powers of 2 and powers of 3
// you could probably produce a nice self similar structure, a la turtle graphcis
// or model tension and release, momentum

// let motif = play, up fifth, wait, play, wait/2, play, wait/2
// motif down tone motif down tone motif down tone
// let music = buffer of sounds, like opengl

// have a process that has an array of "modules" and each one has index of next to jump to, like a turing tape
// turing machine including stochastic decision process for adding more rules
// you could have a supervising process optimizing for something...

// what about do a logistic map or a lorenz attractor or something

// whats the most natural sequence of rules to follow

fn main() -> anyhow::Result<()> {
    let stream = stream_setup_for(sample_next)?;
    stream.play()?;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
    Ok(())
}

// its interesting how the golden ratio sounds pretty consonant
// 1 1 2 3 5 8 13 21 ...
// 1/1 unison, 2/1 octave, 3/2 perfect fifth ... coincidence? we have perfect fourth, major third not included

// do the decision not every sample
// or chance to change
fn sample_next(o: &mut SampleRequestOptions) -> f32 {
    let Phi = (5.0f32.sqrt()) / 2.0; // silver, i used first
    // oh well it was unlimited because we were timesing and diving but something < 0 lol. 
    let phi = (1.0 + 5.0f32.sqrt()) / 2.0;


    // oh gentlest transition is closest in magnitude to 1

    // oh * phi same as / 1 + phi i spose
    
    // let x = Phi; let p = 0.001;
    // let x = 3. / 2.;
    // let x = Phi/2.0; let p = 0.0001; // the aliens are here wtf and it makes such high frequencies, miescehvicz point?

    // let x = phi/2.0; let p = 0.0001;
    // let x = 1.5; let p = 0.00005;
    // let x = 32. / 27.; let p = 0.00005;
    // let x = 1.05946; let p = 0.00005;
    // let x = 9. / 8.; let p = 0.00005;
    // let x = 7./5.; let p = 0.00005;
    // let x = 1.5; let p = 0.0001;
    // let x = 1.0 - phi/32.0; let p = 0.00005;
    
    // let x = 1.0 - phi/1000.0; let p = 0.01;

    // let p = 0.0001;
    // let x = Phi/2.0;
    // let x = Phi * 2.0;
    // let x = PI;
    // let x = 2.0;

    // let x = 1.0001; let p = 0.1;
    // let x = Phi; let p = 0.1; // delicious noise
    // let x = Phi; let p = 0.01; // crushing noiseo
    // let x = PI; let p = 0.01; // not as rich
    // let x = 2.0; let p = 0.01; // bit crushed, harsher highs?
    // let x = 1.001; let p = 0.1; // soi


    let n = 13;

    let seed = o.rng.next_u32();
    o.n_samples = o.n_samples.wrapping_add(1);
    // if chance(khash(seed), p) {
    if o.n_samples % (1 << n) == 0 {
        let x = if chance(seed * 128397127, 0.5) {
            // 3. / 2.
            // 2.0f32.powf(11./12.)
            729./512.
        } else {
            1024./729.
            // 2.0f32.powf(1./12.)
            // 2.0
        };
        if o.f / x >= 50.0 {
            if o.f * x <= 4000.0 {
                if chance(seed, 0.5) {
                    o.f *= x
                } else {
                    o.f /= x
                };
            } else {
                o.f /= x;
            }
        } else {
            o.f *= x;
        }
        o.sample_clock = 0.0;
    }
    o.modf = 1.0;
    o.sample_clock = (o.sample_clock + 1.0) % o.sample_rate;
    (o.sample_clock * o.f * o.modf * 2.0 * std::f32::consts::PI / o.sample_rate).sin() * 0.1
}

pub struct SampleRequestOptions {
    pub sample_rate: f32,
    pub sample_clock: f32,
    pub nchannels: usize,

    pub f: f32,
    pub modf: f32,
    pub rng: Xoshiro256StarStar,
    pub n_samples: u64,

}
impl SampleRequestOptions {
    pub fn new(sample_rate: f32, nchannels: usize) -> SampleRequestOptions {
        SampleRequestOptions {
            sample_rate,
            nchannels,
            
            n_samples: 0,
            sample_clock: 0.0,
            f: 440.0,
            modf: 1.0,
            rng: Xoshiro256StarStar::seed_from_u64(13223559681708962501),
        }
    }
}

pub fn stream_setup_for<F>(on_sample: F) -> Result<cpal::Stream, anyhow::Error>
where
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static + Copy,
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::F32 => stream_make::<f32, _>(&device, &config.into(), on_sample),
        cpal::SampleFormat::I16 => stream_make::<i16, _>(&device, &config.into(), on_sample),
        cpal::SampleFormat::U16 => stream_make::<u16, _>(&device, &config.into(), on_sample),
    }
}

pub fn host_device_setup(
) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

pub fn stream_make<T, F>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    on_sample: F,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static + Copy,
{
    let sample_rate = config.sample_rate.0 as f32;
    let sample_clock = 0f32;
    let nchannels = config.channels as usize;
    let mut request = SampleRequestOptions::new(sample_rate, nchannels);
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            on_window(output, &mut request, on_sample)
        },
        err_fn,
    )?;

    Ok(stream)
}

fn on_window<T, F>(output: &mut [T], request: &mut SampleRequestOptions, mut on_sample: F)
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static,
{
    for frame in output.chunks_mut(request.nchannels) {
        let value: T = cpal::Sample::from::<f32>(&on_sample(request));
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
