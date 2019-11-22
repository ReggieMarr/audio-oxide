//This only handles the portaudio bindings
//Note we should change the audiostream stuff to generic
//but essentially we just need to give it an object which
//contains the data reference and some implementation
//that can be used in the call back to do some dsp
use std::sync::mpsc;
//use rustfft::{FFTplanner, FFT};
use arr_macro::arr;
//use num::complex::Complex;
use std::sync::{Arc, Mutex};
use num::complex::Complex;
//No longer used, this was for opengl visualization
//use glium::*;//{
    // Display,
    // Vertex,
    // Surface,
    // VertexBuffer,
    // Program,
    // DrawParameters,
    // Blend,
// };
extern crate portaudio;
use portaudio::{
    PortAudio,
    Stream,
    NonBlocking,
    Input,
    StreamParameters,
    InputStreamSettings,
    InputStreamCallbackArgs,
    Continue,
};
pub type PortAudioStream = Stream<NonBlocking, Input<f32>>;
use crate::data_stream::audio_stream as local_mod;
use local_mod::common::{
    SAMPLE_RATE,
    NUM_BUFFERS,
    INTERLEAVED,
    BUFF_SIZE,
    CHANNELS,
    GAIN,
    FFT_SIZE,
    MultiBuffer,
    AudioSample,
    AudioStream
};
use local_mod::common::Package;
use local_mod::common::MakeMono;
use local_mod::common::InputHandler;
use std::marker::PhantomData;

impl MakeMono for AudioStream<'_, Complex<f32>, AudioSample> {
    fn make_mono<'a, DataSet, DataMember>(&self, data : &'a [DataSet], time_index :usize)
    //fn make_mono<'a, DataSet, DataMember>(&self, data : [Complex<f32>; FFT_SIZE], time_index :usize)
        ->std::io::Result<DataSet>
        where DataSet : std::ops::Add,
              DataSet : std::ops::Mul
    {
        //should use const but for now we will hard code FFT_SIZE
        static mut sample_buffer : [Complex<f32>; FFT_SIZE] = arr![Complex::new(0.0, 0.0); 1024];
        //should assert that split point is indeed the middle of the buffer
        let (left, right) = sample_buffer.split_at_mut(FFT_SIZE);
        //This takes the buffer input to the stream and then begins describing the
        //input using complex values on a unit circle.alloc
        //let data = data as Vec<f32>;
        for ((x, t0), t1) in data.chunks(CHANNELS)
            .zip(left[time_index..(time_index + BUFF_SIZE)].iter_mut())
            .zip(right[time_index..(time_index + BUFF_SIZE)].iter_mut())
        {
            let mono = Complex::new(GAIN * (x[0] + x[1]) / 2.0, 0.0);
            *t0 = mono;
            *t1 = mono;
        }
        Ok(sample_buffer)
    }
}
//impl InputHandler for AudioStream<'static, Vec<f32>>{};

pub trait SetupStream {
    //fn setup<OutputType, ErrorType>(&self)->Result<OutputType, ErrorType> {
    fn setup<OutputType, ErrorType>(&self)->Result<OutputType, ErrorType> {
    }
}
//pub fn init_audio_simple(config: &Devicecfg) -> Result<(PortAudioStream, MultiBuffer), portaudio::Error> {
//we either wanna pass the audio stream we are implementing our specfics on.
//And we want to pass the address/item which will be mutated here. We wont
//we will only take the return for portaudio errors. Runtime errors can be checked
//panicing here and on our implementation side checking if the mutex has been poisened
impl SetupStream for AudioStream<'_, Complex<f32>, AudioSample> {
    fn setup(&self)->Result<PortAudioStream, portaudio::Error> {
        let pa = PortAudio::new().expect("Unable to init portaudio");

        let def_input = pa.default_input_device().expect("Unable to get default device");
        let input_info = pa.device_info(def_input).expect("Unable to get device info");
        let latency = input_info.default_low_input_latency;
        // Set parameters for the stream settings.
        // We pass which mic should be used, how many channels are used,
        // whether all the values of all the channels should be passed in a
        // single audiobuffer and the latency that should be considered
        let input_params = StreamParameters::<f32>::new(def_input, CHANNELS as i32, INTERLEAVED, latency);

        pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
        // Settings for an inputstream.
        // Here we pass the stream parameters we set before,
        // the sample rate of the mic and the amount values we want
        let settings = InputStreamSettings::new(input_params, SAMPLE_RATE, BUFF_SIZE as u32);

        //let mut audio_buffer : [[Mutex<AudioSample>; BUFF_SIZE]; NUM_BUFFERS];
        //This creates a thread safe reference counting pointer to buffers
        //Safe Audio Reference (SAR)
        //let sar_buff = Arc::new(audio_buffer);
        // This is a lambda which I want called with the samples
        let (receiver, callback) = {
            let (sender, receiver) = mpsc::channel();
            //let local_sar = sar_buff.clone();
            //let stream_handler = AudioStream::<'_,Vec<f32>>::new(local_sar);

            //somehow this reads buffer as a module and data as some buffer value
            (receiver, move |InputStreamCallbackArgs { buffer: data, .. }| {
                //it might actually make more sense to just get the data, make it into a mono
                //sample, and then return that as as the buffer
                self.handle_input(data);
                let current_sample = self.peak_current_sample().lock().unwrap();
                if self.package().unwrap() {
                    sender.send(()).ok();
                }
                Continue
            })
        };
        // Registers the callback with PortAudio
        let mut stream = pa.open_non_blocking_stream(settings, callback)?;
        Ok(stream)
    }
}
