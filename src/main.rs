extern crate rand; // use bincode::serde::serialize; // use bincode::serde::serialize; use std::mem::size_of;
//use std::mem::size_of_val;
//use termcolor::{Color, Ansi, ColorChoice, ColorSpec, StandardStream, WriteColor};
//use std::io::Write;
//use std::option;
use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use std::{thread, time};
pub mod device_modules;
//mod audio;
//use audio::{init_audio, init_audio_simple, Vec2, Vec4, SAMPLE_RATE};

use num::complex::Complex;
//use device_modules::config::*;
//use std::io;
//use std::sync::mpsc::*;

//mod visualizer;
//use visualizer::display;

//#[macro_use] extern crate serde_derive;
extern crate serde_json;
// use serde_json;
mod signal_processing;
use signal_processing::{
    TransformOptions,
    Sample,
};
use std::sync::{Arc, Mutex};
use rustfft::{FFTplanner, FFT};
mod pixel;
mod data_stream;
use data_stream::audio_stream as audio_stream;

use audio_stream::{
    common::AudioStream,
    common::AudioSample,
    common::BUFF_SIZE,
    common::NUM_BUFFERS,
    common::FFT_SIZE,
};
use audio_stream::{
    audio_source::StartStream,
    audio_source::PortAudioStream,
};

use audio_stream::common::Process;
use audio_stream::common::ADCResolution;
use audio_stream::common::InputStreamSample;
use audio_stream::common::InputStreamSlice;
use audio_stream::common::AudioSampleStream;
use std::io::Result;

impl Process for AudioStream<AudioSampleStream> {
    fn process(&self, input : &mut InputStreamSample)->Result<InputStreamSample> {
        //should be able to derive this from SourceType
        static mut fft_planner : FFTplanner<ADCResolution> = FFTplanner::new(false);
        static fft : Arc<dyn FFT<ADCResolution>> = fft_planner.plan_fft(FFT_SIZE);
        static mut output : InputStreamSample = Vec::with_capacity(FFT_SIZE);
        fft.process(input, &mut output);
        Ok(output)
    }
}

fn main() -> std::io::Result<()> {

    //Start the audio stream
    //we pass the lifetime of the current scope into audiostream so that
    //it will stay alive even if we say, end a stream
    let tune_stream : AudioStream<AudioSampleStream> = AudioStream::default();
    let mut stream : &'_ PortAudioStream = tune_stream.startup().unwrap();

    stream.start().expect("Unable the open stream");
    thread::sleep(time::Duration::from_secs(10));
    let handle = thread::spawn(move || {
        let mut buff_idx = 0;
        let mut sample_idx = 0;
        let mut first = false;
        let spectrum_index = 0;
        loop {
            let buffers = tune_stream.buffers[buff_idx][sample_idx].lock().unwrap();
            //loop {
            //    let buffer_sample = buffers.lock().unwrap();
            //    let mut buffer = buffer_sample[spectrum_index];
            //}
            //This is 258 since it needs to store the full range + 3 values to maintain
            //Continuit y
            //do something like this
            //buff_idx = (buff_idx + 1) % buffers.len();
            //here we borrow a reference to buffer.analytic
            //this allows get_freq_chart to use the data but ensure nothing else
            //can manipulate it
            // println!("size is {:?}",buffer.analytic.len());
            //make sure to unwrap Results to properly iterate
            //let freq_mag = get_freq_chart(&buffer.analytic, fft_size, false).unwrap();
        }
    });
    handle.join().unwrap();
    stream.stop();

    Ok(())
}
