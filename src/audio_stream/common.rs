use num::complex::Complex;

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
pub type PortAudioStream = Stream<NonBlocking, Input<f32>>;
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

//The only reason we really need this is the rendered
//attribute. Dropping for now
//struct AudioBuffer {
//    pub rendered: bool,
//    pub analytic: Vec<AudioSample>,
//}


//TODO consider creating a more generic samplestream that
//we can make into an audiostream
pub struct AudioStream<'stream_life, DataStreamType> {
    //buffer                  : Arc<Vec<AudioBuffer>>,
    buffer                  : Arc<[[AudioSample; BUFF_SIZE]; NUM_BUFFERS]>,
    //TODO possibly encapsulate this stuff as its own thing
    thalweg                 : Sample<'stream_life, DataStreamType, AudioSample>,
}

pub trait Package {
    fn package<R>(&self)->std::io::Result<R>;
}


pub trait New {
    fn new<CFGTYPE>(&self, cfg_data : CFGTYPE)->Self;
}


