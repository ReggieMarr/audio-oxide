// extern crate rand; // use bincode::serde::serialize; // use bincode::serde::serialize; use std::mem::size_of;
//use std::mem::size_of_val;
//use termcolor::{Color, Ansi, ColorChoice, ColorSpec, StandardStream, WriteColor};
//use std::io::Write;
//use std::option;
//use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use std::{thread, time};
//pub mod device_modules;
//mod audio;
//use audio::{init_audio, init_audio_simple, Vec2, Vec4, SAMPLE_RATE};

use num::complex::Complex;
//use device_modules::config::*;

//mod visualizer;
//use visualizer::display;

//#[macro_use] extern crate serde_derive;
extern crate serde_json;
// use serde_json;
mod signal_processing;
use std::sync::Arc;
use rustfft::{FFTplanner, FFT};
mod pixel;
mod data_stream;
use data_stream::audio_stream as audio_stream;

use audio_stream::{
    common::AudioStream,
    common::FFT_SIZE,
    common::NUM_BUFFERS
};

use audio_stream::common::Process;
use audio_stream::common::ADCResolution;
use audio_stream::common::InputStreamSample;
use audio_stream::common::AudioSampleStream;
use std::io::Result;
use audio_stream::audio_source::startup;

impl Process for AudioStream<AudioSampleStream> {
    fn process(&self, mut input : InputStreamSample, time_index : usize)->Result<InputStreamSample> {
        //should be able to derive this from SourceType
        if input.len() < FFT_SIZE {
            panic!("input should be size {:?} is size {:?}", FFT_SIZE, input.len());
        }
        let mut fft_planner : FFTplanner<ADCResolution> = FFTplanner::new(false);
        let fft : Arc<dyn FFT<ADCResolution>> = fft_planner.plan_fft(FFT_SIZE);
        let mut output : InputStreamSample = vec![Complex::new(0.0, 0.0); FFT_SIZE];
        fft.process(&mut input[time_index..(time_index + FFT_SIZE)], &mut output[..]);
        Ok(output)
    }
}

fn main() -> std::io::Result<()> {

    //Start the audio stream
    let mut stream_handler : AudioStream<AudioSampleStream> = AudioStream::default();
    let startup_res = startup(&mut stream_handler).unwrap();
    let mut stream = startup_res.stream;
    let receiver = startup_res.receiver;

    stream.start().expect("Unable the open stream");
    thread::sleep(time::Duration::from_secs(10));
    let handle = thread::spawn(move || {
        let mut _sample_idx = 0;
        let mut _first = false;
        let _spectrum_index = 0;
        loop {
            let received = receiver.recv().unwrap();
            for buff_idx in 0..NUM_BUFFERS {
                let sample = received.buffers[buff_idx].lock().unwrap();
                // for data in &sample.data_points {
                //     if data.sample_freq == 0.0 {
                //         println!("{:?} {:?} {:?}", received.rendered[buff_idx], data.sample_freq, data.complex_point);
                //     }
                //     else if received.rendered[buff_idx] {
                //         println!("Rendered val");
                //     }
                // }
            }
        }
    });
    handle.join().unwrap();
    stream.stop();
    Ok(())
}
