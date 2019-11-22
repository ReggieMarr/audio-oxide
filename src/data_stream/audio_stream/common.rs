use num::complex::Complex;
use std::sync::{Arc, Mutex};
pub const SAMPLE_RATE : f64   = 44_100.0;
pub const CHANNELS        : usize   = 2;
pub const INTERLEAVED     : bool  = true;
pub const NUM_BUFFERS     : usize = 256;
pub const BUFF_SIZE       : usize = 256;
//These should be stored in some sort of config file/object
pub const ANGLE_CUTOFF    : f32   = 0.01;
pub const ANGLE_Q         : f32   = 0.5;
pub const NOISE_CUTOFF    : f32   = 0.01;
pub const NOISE_Q         : f32   = 0.5;
pub const FFT_SIZE        : usize = 1024;
pub const GAIN            : f32   = 1.0;

//These could probably be combined and handled as their own struct
pub type MultiBuffer = Arc<[[Mutex<AudioSample>; BUFF_SIZE]; NUM_BUFFERS]>;
// pub type ReceiveType = mpsc::Receiver<mpsc>;

use crate::signal_processing::{Sample, TransformOptions};

pub struct AudioSample {
    //a single point on a complex unit circle
    complex_point : Complex<f32>,
    //the frequency of the point of the unit circle
    sample_freq : f32,
    //average angular noise
    angular_noise : f32,
    //optionally we could also add angular velocity here
    angular_velocity : f32,
}

impl Default for AudioSample {
    fn default()->Self {
        AudioSample {
            complex_point : Complex::new(0.0f32, 0.0f32),
            sample_freq : 0.0f32,
            angular_noise : 0.0f32,
            angular_velocity : 0.0f32,
        }
    }
}

pub trait Package {
    fn package<R>(&self)->std::io::Result<R>;
}

pub trait MakeMono {
    //this will be normalize
    //it might make more sense to set this in the def of the trait
    fn make_mono<'a, DataSet, DataMember>(&self, data : &'a [DataSet], time_index : usize)->std::io::Result<DataSet>;
    //fn make_mono<'a, DataSet, DataMember>(&self, data : [Complex<f32>; FFT_SIZE], time_index: usize)->std::io::Result<DataSet>;
}

pub trait InputHandler {
    //it might make more sense to set this in the def of the trait
    type DataSet;
    fn handle_input<DataType>(&self,data : DataType)->std::io::Result<usize>;
}

impl<StructType> InputHandler for StructType
    where StructType : MakeMono {
    fn handle_input<DataType>(&self,data : DataType)->std::io::Result<usize> {
        //this updates the time index as we continue to sample the audio stream
        static mut time_index : usize = 0;
        self.make_mono(data, time_index)?;
        time_index = ((time_index + BUFF_SIZE) % FFT_SIZE).try_into().unwrap();
        Ok(time_index)
    }
}

pub trait New {
    fn new<CFGTYPE>(&self, cfg_data : CFGTYPE)->Self;
}
//TODO consider creating a more generic samplestream that
//we can make into an audiostream
//#[derive(InputHandler)]
pub struct AudioStream<'stream_life, ADC, BufferT> {
    //Using sample probably adds more overhead than needed but lets just try
    pub buffer     : Arc<[[Mutex<Sample<'stream_life, ADC, BufferT>>; BUFF_SIZE]; NUM_BUFFERS]>,
    //these should be private and immutable
    pub current_buff   : usize,
    pub current_sample : usize,
    //pub thalweg                 : Sample<'stream_life, DataStreamType, AudioSample>,
}
