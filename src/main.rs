extern crate rand; // use bincode::serde::serialize; // use bincode::serde::serialize; use std::mem::size_of;
use std::mem::size_of_val; use termcolor::{Color, Ansi, ColorChoice, ColorSpec, StandardStream, WriteColor}; use std::io::Write; use std::option;
use rand::Rng;
use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use std::{thread, time};
use std::vec::Vec;
pub mod device_modules;
mod audio;
use audio::{init_audio, init_audio_simple, Vec2, Vec4, SAMPLE_RATE};

use num::complex::Complex;
use device_modules::config::*;
use std::io;
use std::sync::mpsc::*;

mod visualizer;
use visualizer::display;

use std::f64;
use std::f32;
use std::mem;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
// use serde_json;
mod signal_processing;
use signal_processing::{
    TransformOptions,
    Sample,
};
use std::sync::{Arc, Mutex};
use rustfft::{FFTplanner, FFT};
impl<SourceType> TransformOptions<SourceType> for Sample<'_, SourceType, SourceType> {
    fn process<TransformBaseType>(&self) {
        //should be able to derive this from SourceType
        type TransformBaseType = f32;
        static mut fft_planner : FFTplanner<TransformBaseType> = FFTplanner::new(false);
        static fft : Arc<dyn FFT<TransformBaseType>> = fft_planner.plan_fft(FFT_SIZE);
        fft.process(self.data_points,self.data_points);
    }

}

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
    audio_source::SetupStream,
    audio_source::PortAudioStream,
};

impl<'a, ADC, BufferT> Default for AudioStream<'a, ADC, BufferT> {

    fn default()->Self {
    //fn make_audio_stream<'a, ADC, BufferT>()->AudioStream<'a, ADC, BufferT> {
    //fn make_audio_stream<'a, ADC>()->AudioStream<'a, ADC, AudioSample> {
        /*
           we create a sample which takes in the type which is the output of the InputHandler
           (this will be [Complex<f32>;FFT_SIZE], but also requires type [Complex<f32>; FFT_SIZE*2])
           The sample then calls its process method and then after this is called the package method
           is called which gives us our output which is [AudioSample; BUFF_SIZE + 3]
           this gives us one audio buffer of which we will make 3 of before sending the RollingSample
           buffer sample
        */
        //type AudioSample = Sample<'a, ADC, BufferT>;
        let mut buffer : [[ Mutex<Sample<'a, ADC, BufferT>>; BUFF_SIZE]; NUM_BUFFERS];
        for outer in 0..NUM_BUFFERS {
            for inner in 0..BUFF_SIZE {
                buffer[outer][inner] = Mutex::new(Sample::default());
            }
        }
        let safe_buffer = Arc::new(buffer);
        AudioStream{
            buffer : safe_buffer,
            current_buff : 0,
            current_sample : 0,
        }
    }
}

fn main() -> std::io::Result<()> {

    //Start the audio stream
    //we pass the lifetime of the current scope into audiostream so that
    //it will stay alive even if we say, end a stream
    let tune_stream : AudioStream<'_, Complex<f32>, AudioSample> = AudioStream::default();
    let mut stream : PortAudioStream = tune_stream.setup().unwrap();
    //let (mut stream, buffers) = init_audio_simple(&esp_if).unwrap();
    // let (mut stream, buffers) = init_audio(&esp_if).unwrap();

    stream.start().expect("Unable the open stream");
    thread::sleep(time::Duration::from_secs(10));
    let handle = thread::spawn(move || {
        let mut index = 0;
        // for _ in (0..1024-259) {
            while !buffers[index].lock().unwrap().rendered {
                let mut buffer = buffers[index].lock().unwrap();
                //This is 258 since it needs to store the full range + 3 values to maintain
                //Continuity
                // ys_data.copy_from_slice(&buffer.analytic);
                buffer.rendered = true;
                index = (index + 1) % buffers.len();
                //here we borrow a reference to buffer.analytic
                //this allows get_freq_chart to use the data but ensure nothing else
                //can manipulate it
                // println!("size is {:?}",buffer.analytic.len());
                //make sure to unwrap Results to properly iterate
                let freq_mag = get_freq_chart(&buffer.analytic, fft_size, false).unwrap();
                // let pixel_box = make_pixel_packet(&freq_mag.amplitudes, 125).unwrap();
                // let pixel_packet = *pixel_box;
                // send_udp_packet(local_socket, &pixel_packet).unwrap();
            }
        // }
    });
    // display(buffers);
    handle.join().unwrap();
    // stream.stop();

    Ok(())
}

