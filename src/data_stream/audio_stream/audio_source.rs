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
type StreamType = AudioStream<AudioSampleStream>;
pub struct StreamInterface<'a> {
    pub receiver : mpsc::Receiver<&'a StreamType>,
    pub stream : PortAudioStream
}

pub fn startup<'a>()->Result<StreamInterface<'a>, portaudio::Error> {
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

    // This is a lambda which I want called with the samples
    let (receiver, callback) = {
        let (sender, receiver) = mpsc::channel::<&'a StreamType>();
        (receiver, move |InputStreamCallbackArgs { buffer : mut data, .. }| {
            let mut stream_handler : AudioStream<AudioSampleStream> = AudioStream::default();
            let handle_res = stream_handler.handle_input(&mut data).unwrap();
            let processed_buffer = stream_handler.process(handle_res.0, handle_res.1).unwrap();
            if stream_handler.package(processed_buffer).unwrap() {
                sender.send(&stream_handler).ok();
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
