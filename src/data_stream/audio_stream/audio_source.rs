//This only handles the portaudio bindings
//Note we should change the audiostream stuff to generic
//but essentially we just need to give it an object which
//contains the data reference and some implementation
//that can be used in the call back to do some dsp
use std::sync::mpsc;
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
    INTERLEAVED,
    BUFF_SIZE,
    CHANNELS,
    AudioStream
};
use local_mod::common::Package;
use local_mod::common::InputHandler;
use local_mod::common::ADCResolution;
use local_mod::common::AudioSampleStream;
use local_mod::common::Process;
pub struct StreamInterface<'a> {
    pub receiver : mpsc::Receiver<&'a mut AudioStream<AudioSampleStream>>,
    pub stream : PortAudioStream
}
//we either wanna pass the audio stream we are implementing our specfics on.
//And we want to pass the address/item which will be mutated here. We wont
//we will only take the return for portaudio errors. Runtime errors can be checked
//panicing here and on our implementation side checking if the mutex has been poisened
// pub fn startup()->Result<(PortAudioStream, mpsc::Receiver<AudioStream<AudioSampleStream>>), portaudio::Error> {
pub fn startup<'a>(stream_handler : &'a mut AudioStream<AudioSampleStream>)->Result<StreamInterface, portaudio::Error> {
    let pa = PortAudio::new().expect("Unable to init portaudio");

    let def_input = pa.default_input_device().expect("Unable to get default device");
    let input_info = pa.device_info(def_input).expect("Unable to get device info");
    let latency = input_info.default_low_input_latency;
    // Set parameters for the stream settings.
    // We pass which mic should be used, how many channels are used,
    // whether all the values of all the channels should be passed in a
    // single audiobuffer and the latency that should be considered
    let input_params = StreamParameters::<ADCResolution>::new(def_input, CHANNELS as i32, INTERLEAVED, latency);

    pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
    // Settings for an inputstream.
    // Here we pass the stream parameters we set before,
    // the sample rate of the mic and the amount values we want
    let settings = InputStreamSettings::new(input_params, SAMPLE_RATE, BUFF_SIZE as u32);

    // let mut stream_handler : AudioStream<AudioSampleStream> = AudioStream::default();
    //This creates a thread safe reference counting pointer to buffers
    // This is a lambda which I want called with the samples
    let (receiver, callback) = {
        let (sender, receiver) = mpsc::channel::<&'a mut AudioStream<AudioSampleStream>>();
        (receiver, move |InputStreamCallbackArgs { buffer : mut data, .. }| {
            let handle_res = stream_handler.handle_input(&mut data).unwrap();
            let processed_buffer = stream_handler.process(handle_res.0, handle_res.1).unwrap();
            //let current_sample = self.peak_current_sample().lock().unwrap();
            if stream_handler.package(processed_buffer).unwrap() {
                sender.send(stream_handler).ok();
            }
            Continue
        })
    };
    let startup_res = StreamInterface {
        receiver : receiver,
        // Registers the callback with PortAudio
        stream : pa.open_non_blocking_stream(settings, callback)?
    };
    Ok(startup_res)
}