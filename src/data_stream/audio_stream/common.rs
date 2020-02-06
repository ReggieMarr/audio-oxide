use num::complex::Complex;
use std::sync::{Arc, Mutex};
pub const SAMPLE_RATE : f64   = 44_100.0;
pub const CHANNELS        : usize   = 2;
pub const INTERLEAVED     : bool  = true;
// pub const NUM_BUFFERS     : usize = 256;
pub const NUM_BUFFERS     : usize = 16;
pub const BUFF_SIZE       : usize = 256;
//These should be stored in some sort of config file/object
pub const ANGLE_CUTOFF    : f32   = 0.01;
pub const ANGLE_Q         : f32   = 0.5;
pub const NOISE_CUTOFF    : f32   = 0.01;
pub const NOISE_Q         : f32   = 0.5;
pub const FFT_SIZE        : usize = 1024;
pub const GAIN            : f32   = 1.0;

//These could probably be combined and handled as their own struct
// pub type ReceiveType = mpsc::Receiver<mpsc>;

use crate::signal_processing::Sample;
use crate::signal_processing::Scope;
use std::io::Result;

#[derive(Copy, Clone, Default, Debug)]
pub struct AudioSample<Resolution> {
    //a single point on a complex unit circle
    pub complex_point : Complex<Resolution>,
    //the frequency of the point of the unit circle
    pub sample_freq : Resolution,
    //average angular noise
    pub angular_noise : Resolution,
    //optionally we could also add angular velocity here
    pub angular_velocity : Resolution,
}

//pub type MultiBuffer = Arc<[[Mutex<AudioSample>; BUFF_SIZE]; NUM_BUFFERS]>;
pub type ADCResolution= f32;
pub type InputStreamSample = Vec<Complex<ADCResolution>>;
pub type InputStreamSlice<'a> = &'a [ADCResolution];
pub type AudioSampleStream = AudioSample<ADCResolution>;
//pub type AudioSampleStream = Vec<AudioSample<ADCResolution>>;
pub type AudioBuffer = Arc<[Mutex<Sample<[AudioSampleStream; BUFF_SIZE]>>; NUM_BUFFERS]>;

pub trait New {
    fn new<CFGTYPE>(&self, cfg_data : CFGTYPE)->Self;
}
//TODO consider creating a more generic samplestream that
//we can make into an audiostream

#[derive(Clone)]
pub struct AudioStream<AudioSampleStream> {
    //pub buffers : Arc<[Mutex<Sample<[AudioSampleStream; BUFF_SIZE]>>; NUM_BUFFERS]>,
    //pub buffers : Arc<Vec<Mutex<Sample<[AudioSampleStream; BUFF_SIZE]>>>>,
    pub buffers : Arc<Vec<Mutex<Sample<Vec<AudioSampleStream>>>>>,
    //these should be private and immutable
    pub rendered : [bool; NUM_BUFFERS],
    pub current_buff   : usize,
    pub current_sample : usize,
}

impl Default for AudioStream<AudioSampleStream> {

    fn default()->Self {
        /*
           we create a sample which takes in the type which is the output of the InputHandler
           (this will be [Complex<f32>;FFT_SIZE], but also requires type [Complex<f32>; FFT_SIZE*2])
           The sample then calls its process method and then after this is called the package method
           is called which gives us our output which is [AudioSample; BUFF_SIZE + 3]
           this gives us one audio buffer of which we will make 3 of before sending the RollingSample
           buffer sample
        */

        //let mut cfg_buffers : [Mutex<[Sample<AudioSampleStream>; BUFF_SIZE]>; NUM_BUFFERS];
        //let cfg_buffer : Sample<[AudioSampleStream; BUFF_SIZE]> = Sample::new(Option::None, Option::None);
        let mut cfg_samples : Vec<AudioSampleStream> = Vec::with_capacity(BUFF_SIZE);
        for _ in 0..BUFF_SIZE {
            cfg_samples.push(AudioSampleStream::default());
        }
        //let cfg_buffers : Vec<Mutex<Sample<[AudioSampleStream; BUFF_SIZE]>>> =
        let mut cfg_buffers : Vec<Mutex<Sample<Vec<AudioSampleStream>>>> =
            Vec::with_capacity(NUM_BUFFERS);
        let cfg_scope = Scope::new(0, BUFF_SIZE);
        for _ in 0..NUM_BUFFERS {
            //TODO make this not clone
            cfg_buffers.push(Mutex::new(Sample::new(Some(cfg_samples.clone()), Some(cfg_scope))));
        }
        //let cfg_buffers : [Mutex<Sample<[AudioSampleStream; BUFF_SIZE]>>; NUM_BUFFERS] =
        //    [Mutex::new(Sample::new(Option::None, Option::None)); NUM_BUFFERS];
        //for outer in 0..NUM_BUFFERS {
        //    //for inner in 0..BUFF_SIZE {
        //    //    //This should really allow some sort of passing const so I can say the
        //    //    //size of the array
        //    //    cfg_buffer[inner] = Sample::new(Option::None, Option::None, Some(FFT_SIZE));
        //    //}
        //    //cfg_buffer = Sample::new(Option::None, Option::None);
        //    //cfg_buffers[outer] = Mutex::new(cfg_buffer);
        //    cfg_buffers[outer] = Mutex::new(Sample::new(Option::None, Option::None));
        //}
        let buffers_ref = Arc::new(cfg_buffers);
        AudioStream{
            buffers : buffers_ref,
            rendered : [false; NUM_BUFFERS],
            current_buff : 0,
            current_sample : 0,
        }
    }
}

pub trait InputHandler {
    //it might make more sense to set this in the def of the trait
    //type DataSet;
    fn handle_input(&self, raw_data : &mut InputStreamSlice)->std::io::Result<(InputStreamSample, usize)>;
}

impl InputHandler for AudioStream<AudioSampleStream>
    where AudioStream<AudioSampleStream> : MakeMono<InputStreamSample> {
    fn handle_input(&self, raw_data : &mut InputStreamSlice)->Result<(InputStreamSample, usize)>
    {
        //this updates the time index as we continue to sample the audio stream
        static mut TIMEINDEX : usize = 0;
        unsafe {
            //due to mutable reference of static, we should work around this
            //using some explicit lifetime
            let unified_buffer = self.make_mono(raw_data, TIMEINDEX).unwrap();
            TIMEINDEX = (TIMEINDEX + BUFF_SIZE) % FFT_SIZE;
            Ok((unified_buffer, TIMEINDEX))
        }
        // Ok(unified_buffer[time_index..(time_index + FFT_SIZE)].to_vec())
    }
}

pub trait MakeMono<InputStreamSample> {
    //this will be normalize
    //it might make more sense to set this in the def of the trait
    fn make_mono(&self, data : &mut InputStreamSlice, time_index : usize)->Result<InputStreamSample>;
    //fn make_mono<DataSet, DataMember>(&self, data : &[DataSet], time_index : usize)->std::io::Result<DataSet>;
}

impl MakeMono<InputStreamSample> for AudioStream<AudioSampleStream> {
    //fn make_mono(&self, data : &mut InputStreamSample, time_index :usize)->Result<InputStreamSample>
    fn make_mono(&self, data : &mut InputStreamSlice, time_index :usize)->Result<InputStreamSample>
    {
        //should use const but for now we will hard code FFT_SIZE
        //static mut sample_buffer : [Complex<f32>; FFT_SIZE] = arr![Complex::new(0.0, 0.0); 1024];
        let mut sample_buffer : InputStreamSample = vec![Complex::new(0.0, 0.0); 2 * FFT_SIZE];
        //should assert that split point is indeed the middle of the buffer
        let (left, right) = sample_buffer.split_at_mut(FFT_SIZE);
        //This takes the buffer input to the stream and then begins describing the
        //input using complex values on a unit circle.alloc
        //let data = data as Vec<f32>;
        for ((x, t0), t1) in data.chunks(CHANNELS)
            .zip(left[time_index..(time_index + BUFF_SIZE)].iter_mut())
            .zip(right[time_index..(time_index + BUFF_SIZE)].iter_mut())
        {
            if x[0] + x[1] == 0.0 {
                println!("Input is 0!");
            }
            let mono = Complex::new(GAIN * (x[0] + x[1]) / 2.0, 0.0f32);
            *t0 = mono;
            *t1 = mono;
        }
        Ok(sample_buffer)
    }
}

pub trait Process {
    fn process(&self, input : InputStreamSample, time_index : usize)->Result<InputStreamSample>;
}

pub trait Package<ADC, BufferT> {
    fn package(&mut self, package_item : InputStreamSample)->Result<bool>;
}
