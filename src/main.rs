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
use signal_processing::TransformOptionsTrait;
mod pixel;
mod audio_stream;
use audio_stream::audio_source::startup_audio_stream;
use audio_stream::default_implementer::*;

#[macro_use]
extern crate lazy_static;

//use audio_stream::Scalar;
//use audio_stream::callbacks;

// mod plotting;
// use plotting::*;

#[allow(unreachable_code)]
fn setup_device(cfg_settings:&mut Devicecfg) -> (StatusType) {
    let mut input = String::new();
    let mut cfg_complete = false;

    println!("Setup custom config: s");
    println!("Use Default settings: d");
    println!("Quit: q");
    while !cfg_complete {
        match io::stdin().read_line(&mut input) {
            Ok(_number_of_bytes) => {
                match input.trim() {
                    "s" => {unimplemented!()},
                    "d" => {
                        cfg_complete = true;
                        *cfg_settings =  Devicecfg::default();
                    }
                    _  => {
                        panic!("Unhandled case")
                    },

                };
            }
            Err(error) => println!("error: {}", error),
        }
    }
    return StatusType::ERROR;
}

fn config_mode() {
    let mut device_settings = Devicecfg::default();
    setup_device(& mut device_settings);
    println!("Setup Remote Device ? y/n");
    //TODO make this a macro
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_number_of_bytes) => {
            match input.trim() {
                "y" => {unimplemented!()},
                "n" => {
                },
                _  => {
                    panic!("Unhandled case")
                },

            };
        }
        Err(error) => println!("error: {}", error),
    }

}

#[derive(Debug)]
struct FrequencyBuff {
    frequencies : Vec<f32>,
    amplitudes : Vec<f32>
}

fn get_freq_chart(audio_buff : &Vec<Vec4>, vec_size : usize, use_polar : bool) -> std::io::Result<(FrequencyBuff)> {
    let mut freq_buff = FrequencyBuff {
        frequencies : Vec::with_capacity(audio_buff.len()),
        amplitudes : Vec::with_capacity(audio_buff.len())
    };
    for audio_packet in audio_buff.iter() {
    let real_part = audio_packet.vec[0];
    let im_part = audio_packet.vec[1];
        //Unused for now
        let freq = audio_packet.vec[2];
        // let ang_velocity = audio_packet.vec[2];
        // let ang_noise = audio_packet.vec[3];
        freq_buff.frequencies.push(freq);
        if use_polar {
            let mag_polar = f32::sqrt(real_part.exp2() + im_part.exp2());
                let mag_db_polar = 20.0f32*(2.0f32*mag_polar/vec_size as f32).abs().log10();
                if mag_db_polar.is_infinite() {
                    freq_buff.amplitudes.push(0.0f32);
                }
                else {
                    freq_buff.amplitudes.push(mag_db_polar);
                }
        } else {
                let mag_db_rect = 20.0f32*(((2.0f32*im_part/vec_size as f32).abs()).log10());
                if mag_db_rect.is_infinite() {
                    freq_buff.amplitudes.push(0.0f32);
                }
                else {
                    freq_buff.amplitudes.push(mag_db_rect);
                }
        }
    }
    Ok(freq_buff)
}

fn main() -> std::io::Result<()> {
    {
        let esp_if = Devicecfg::default();
        // let esp_addr = SocketAddr::new(esp_if.device_specific_cfg.udp_ip, esp_if.device_specific_cfg.udp_port);
        let esp_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5005);
        // let init_strip = make_random_led_vec(25);
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        // writeln!(&mut stdout, "green text!");
        let mut box_buff : termcolor::Buffer;// = "█";
        // for led_idx in init_strip {
        //     // let pixel = Pixel::new(led_idx);
        //     stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(led_idx[0], led_idx[1], led_idx[2]))));
        //     println!("▀");
        //     // println!("led_val: {:?}", led_idx);
        //     update_esp8266(esp_addr, &led_idx)?;
        // }

        //get the frequency portion of the frequencyxmagnitude graph
        let fft_size : usize = 1024;

        let freq_res = SAMPLE_RATE as f32/fft_size as f32; //frequency resolution
        let mut freq_vec : Vec<f32> = Vec::with_capacity(fft_size);
        for (bin_idx, _) in (0..fft_size).enumerate(){
            freq_vec.push(bin_idx as f32 * freq_res);
        }
        let local_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7776);
        let arduino_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)), 7777);
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

    }
    Ok(())
}


fn colour_from_vert4(base_hue : f32, decay : f32, desaturation : f32, relative_length : f32, angle : Vec2, position : f32) -> std::io::Result<(f64)> {
    let colour : f64 = 0.0;

    Ok(colour)
}


fn send_udp_packet(socket_address : SocketAddr, esp_packet : &[u8]) -> std::io::Result<()> {
    /*
    The ESP8266 will receive and decode the packets to determine what values
    to display on the LED strip. The communication protocol supports LED strips
    with a maximum of 256 LEDs.

        |i|r|g|b|
    where
        i (0 to 255): Index of LED to change (zero-based)
        r (0 to 255): red value of LED
    The packet encoding scheme is:
        g (0 to 255): green value of LED
        b (0 to 255): blue value of LED
    */
    {
        let local_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5010);
        let socket = UdpSocket::bind(local_address)?;
        socket.send_to(esp_packet, socket_address)?;
    }
    Ok(())
}
