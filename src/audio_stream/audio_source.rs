
use std::sync::mpsc;
use rustfft::{FFTplanner, FFT};
use num::complex::Complex;
use glium::*;//{
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
mod common;
use common::{
    SAMPLE_RATE,
    NUM_BUFFERS,
    INTERLEAVED,
    BUFF_SIZE,
    SAMPLE_RATE,
};


//pub fn init_audio_simple(config: &Devicecfg) -> Result<(PortAudioStream, MultiBuffer), portaudio::Error> {
//we either wanna pass the audio stream we are implementing our specfics on.
//And we want to pass the address/item which will be mutated here. We wont
//we will only take the return for portaudio errors. Runtime errors can be checked
//panicing here and on our implementation side checking if the mutex has been poisened
pub fn startup_audio_stream()->Result<(PortAudioStream, MultiBuffer), portaudio::Error> {
    let pa = PortAudio::new().expect("Unable to init portaudio");

    let def_input = pa.default_input_device().expect("Unable to get default device");
    let input_info = pa.device_info(def_input).expect("Unable to get device info");
    // println!("Default input device name: {}", input_info.name);

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

    let mut audio_buffer : [[Mutex<AudioSample>; BUFF_SIZE]; NUM_BUFFERS];

    //may want to replace this with a map call
    for buff_idx in 0..NUM_BUFFERS {
        for sample_idx in 0..BUFF_SIZE {
            audio_buffer[buff_idx][sample_idx] = Mutex::new(AudioSample::default());
        }
    }
    //This creates a thread safe reference counting pointer to buffers
    //Safe Audio Reference (SAR)
    let sar_buff = Arc::new(audio_buffer);
    // This is a lambda which I want called with the samples
    let (receiver, callback) = {
        let (sender, receiver) = mpsc::channel();
        let local_sar = sar_buff.clone();

        let stream_handler = AudioStream::<'_,Vec<f32>>::new(local_sar);

        //somehow this reads buffer as a module and data as some buffer value
        (receiver, move |InputStreamCallbackArgs { buffer: data, .. }| {
            let stream_output : AudioSample = stream_handler.coalece(data).unwrap();
            if stream_handler.package().unwrap() {
                sender.send(()).ok();
            }
            Continue
        })
    };

    // Registers the callback with PortAudio
    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    Ok((stream, sar_buff))
}

