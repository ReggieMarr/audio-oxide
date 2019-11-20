extern crate rand; // use bincode::serde::serialize; // use bincode::serde::serialize; use std::mem::size_of;
use std::mem::size_of_val; use termcolor::{Color, Ansi, ColorChoice, ColorSpec, StandardStream, WriteColor}; use std::io::Write; use std::option;
use rand::Rng;
use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use std::{thread, time};
use std::vec::Vec;
pub mod device_modules;
mod audio;
use audio::{init_audio, init_audio_simple, Vec2, Vec4, SAMPLE_RATE};

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
mod pixel;
mod data_stream;
use data_stream::audio_stream as audio_stream;

use audio_stream::{
    common::AudioStream,
    common::FFT_SIZE,
};

#[macro_use]
extern crate lazy_static;

fn make_transform_vec<SourceType>()->Vec<Box<dyn Fn(& mut [SourceType; FFT_SIZE])>> {

}

fn main() -> std::io::Result<()> {
    //Create our audio_stream obj
    let tune_stream = AudioStream::new();

    //Start the audio stream
    let (mut stream, buffers) = init_audio_simple(&esp_if).unwrap();
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

