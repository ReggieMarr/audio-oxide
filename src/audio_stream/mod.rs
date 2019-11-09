use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
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
use rustfft::{FFTplanner, FFT};
use glium::*;//{
    // Display,
    // Vertex,
    // Surface,
    // VertexBuffer,
    // Program,
    // DrawParameters,
    // Blend,
// };

use crate::device_modules::config::*;
use num::complex::Complex;
pub mod callbacks;

pub type MultiBuffer = Arc<Vec<Mutex<AudioBuffer>>>;
pub type PortAudioStream = Stream<NonBlocking, Input<f32>>;
// pub type ReceiveType = mpsc::Receiver<mpsc>;


struct AudioPoint {
    //a single point on a complex unit circle
    complex_point : Complex<f32>,
    //the frequency of the point of the unit circle
    sample_freq : f32,
    //average angular noise
    angular_noise : f32,
    //optionally we could also add angular velocity here
    angulat_velocity : f32,
}

struct AudioSample {
    // buffer : Vec<AudioPoint>
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
    fn default() {
        AudioStream {
            complex_point : Complex::new(0.0f32, 0.0f32),
            sample_freq : 0.0f32,
            angular_noise : 0.0f32,
            angular_velocity : 0.0f32,
        }
    }
}

pub struct AudioBuffer {
    pub rendered: bool,
    pub analytic: Vec<AudioSample>,
}

pub const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: i32 = 2;
const INTERLEAVED: bool = true;

//TODO determine if this is really neccessary
use std::convert::TryInto;
use crate::signal_processing::Sample;

// struct Thalweg<SourceType, MouthType> {
//     source : Sample<'static, SourceType, MouthType>

// }

// impl<SourceType, MouthType> Thalweg<SourceType, MouthType> {
//     fn new() {

//     }
//     fn coalece(&self) {

//     }
// }

//TODO consider creating a more generic samplestream that
//we can make into an audiostream
struct AudioStream {
    time_index: u32,
    gain: f32,
    buffer_size : usize,
    buffer_index : usize,
    channels : usize,
    //TODO possibly encapsulate this stuff as its own thing
    thalweg : Sample<'static, Complex<f32>, AudioSample>,
    fft_size : usize,
    //maybe we should just implement fft on audiostream
    transform : Option<Arc<FFT<f32>>>,
    inverse_transform : Option<Arc<FFT<f32>>>,
    filter : Option<Vec<Complex<f32>>>,
    time_ring_buffer : Vec<Complex<f32>>,
    complex_freq_buffer : Vec<Complex<f32>>
}
//http://www.texasthestateofwater.org/screening/html/gloassary.html
/*
Thalweg: The river's longitudinal section, or the line joining the
deepest point in the channel at each stage from source to mouth.
*/

impl AudioStream {
    fn new(&self) -> AudioStream {

    }
    //May want to encapsulate some of the arguments here. Additionally we are breaking the function does one thing rule
    //We actually update the time index and the sample buffer
    fn clean_stream(&self, sample_buffer : &Vec<Complex<f32>>, data : Vec<f32>) {
        //should assert that split point is indeed the middle of the buffer
        let (left, right) = sample_buffer.split_at_mut(self.fft_size);
        //This takes the buffer input to the stream and then begins describing the
        //input using complex values on a unit circle.
        let buff_usize = *self.buffer_size as usize;
        let time_usize_idx = *self.time_index as usize;
        for ((x, t0), t1) in data.chunks(*self.channels)
            .zip(left[time_usize_idx..(time_usize_idx + buff_usize)].iter_mut())
            .zip(right[time_usize_idx..(time_usize_idx + buff_usize)].iter_mut())
        {
            let mono = Complex::new(self.gain * (x[0] + x[1]) / 2.0, 0.0);
            *t0 = mono;
            *t1 = mono;
        }
        //this updates the time index as we continue to sample the audio stream
        *self.time_index = ((time_usize_idx + buff_usize) % self.fft_size).try_into().unwrap();
    }

    fn coalece() {

    }

    // fn process(transform : Option<fn()>, filter : Option<fn()>, inverse_transform : Option<fn()>) {
    fn process(&self) {

        //This represents the amplitude of the signal represented as the distance from the origin on a unit circle
        //Here we transform the signal from the time domain to the frequency domain.
        //Note that humans can only hear sound with a frequency between 20Hz and 20_000Hz
        // fft.process(&mut time_ring_buffer[time_index..time_index + fft_size], &mut complex_freq_buffer[..]);
        if let Some(_) = self.transform {
            let transform_func = self.transform.unwrap();
            transform_func.process(self.time_ring_buffer, self.complex_freq_buffer);
        }

        //the analytic array acts as a filter, removing the negative and dc portions
        //of the signal as well as filtering out the nyquist portion of the signal
        //Also applies the hamming window here

        // By applying the inverse fourier transform we transform the signal from the frequency domain back into the
        if let Some(_) = self.filter {
            let filter_func = self.filter.unwrap();
            filter_func(self.complex_freq_buffer);
            // for (x, y) in analytic.iter().zip(complex_freq_buffer.iter_mut()) {
            //     *y = *x * *y;
            // }
        }
        // By applying the inverse fourier transform we transform the signal from the frequency domain back into the
        // time domain. However now this signal can be represented as a series of points on a unit circle.
        // ifft.process(&mut complex_freq_buffer[..], &mut complex_analytic_buffer[..]);
        if let Some(_) = self.inverse_transform {
            let inverse_func = self.transform.unwrap();
            inverse_func.process(self.time_ring_buffer, self.complex_freq_buffer);
        }
    }

    //leave blank for now but this should be some optional step
    // fn post_process() {
    // }


    fn package(&self) {
        //let mut static analytic_buffer : Vec<AudioSample> = vec![AudioSample::default(); self.buffer_size + 3];
        lazy_static! {
            static ref analytic_buffer : Vec<AudioSample> = vec![AudioSample::default(); self.buffer_size + 3];
        }
        if use_analytic_filt {
            analytic_buffer[0] = analytic_buffer[self.buffer_size];
            analytic_buffer[1] = analytic_buffer[self.buffer_size + 1];
            analytic_buffer[2] = analytic_buffer[self.buffer_size + 2];
        }
        // time domain. However now this signal can be represented as a series of points on a unit circle.
        // ifft.process(&mut complex_freq_buffer[..], &mut complex_analytic_buffer[..]);
        let scale = self.fft_size as f32;
        let freq_res = SAMPLE_RATE as f32 / scale;
        // for (&x, y) in complex_analytic_buffer[fft_size - buffer_size..].iter().zip(analytic_buffer[3..].iter_mut()) {
        //this takes 256 points from the complex_freq_buffer into the analytic_buffer
        // for (&x, y) in complex_freq_buffer[(fft_size - buffer_size)..].iter().zip(analytic_buffer[3..].iter_mut()) {

        let freq_iter = complex_freq_buffer.iter().zip(analytic_buffer.iter_mut());
        for (freq_idx, (&x, y)) in freq_iter.enumerate() {
            let diff = x - prev_input; // vector
            prev_input = x;

            let angle = get_angle(diff, prev_diff).abs().log2().max(-1.0e12); // angular velocity (with processing)
            prev_diff = diff;

            let output = angle_lp(angle);

            let freq_idx = freq_res * freq_idx as f32;

            *y = AudioSample {
                complex_point : x,
                sample_freq : freq_idx,
                noise_lp((angle - output).abs()), // average angular noise
                output.exp2(),
            }
        }

        //what is rendered and why would dropped represent its inverse ?
        let dropped = {
            let mut buffer = buffers[buffer_index].lock().unwrap();
            let rendered = buffer.rendered;
            buffer.analytic.copy_from_slice(&analytic_buffer);
            buffer.rendered = false;
            !rendered
        };
    }
    //TODO, come up with a better name here
    fn check_valve() {

        buffer_index = (buffer_index + 1) % num_buffers;
        if dropped {
            // what does sender do generally ?
            sender.send(()).ok();
        }
        Continue
    }

}


fn callback_function() {
    //open_stream()
    /*
       Starts up the whole process
     */
    //pre_process()
    /*
    Takes some stream, cleans it (optionally)
    and updates process variables
    */
    //process()
    /*
     Takes some stream and does some sort of process
     Right now this goes like
     Transform (optional) -> filter(optional) -> Inverse Transform (optional)
     NOTE: The input type and size of this stream and output is always the same
     */
    //post_process()
    /*
       does some sort of post-processing
       NOTE: The input and output type are the same although the length may not be
    */
    //package()
    /*
     This takes the stream and packages it into some output that will be streamed
     NOTE this will likely change type and size of the input stream
     */
    //finalize()
    //Note: The audio sample is defined as the input given at the process,
    //or post process if there is one, and then the outut of packaging
    //pre_process()

}

pub fn init_audio_simple(config: &Devicecfg) -> Result<(PortAudioStream, MultiBuffer), portaudio::Error> {
    let fft_size = 1024;//config.fft_bins as usize;
    //Found that I had to change the buffer size to 512, not sure if this is really
    //neccessary but for some reason if I don't do this then I only get half the spectrum
    let buffer_size = 256;//config.audio.buffer_size as usize;
    let num_buffers = 16; //config.audio.num_buffers;
    let cutoff = 0.01;
    let q = 0.5;//config.audio.q;
    let pa = PortAudio::new().expect("Unable to init portaudio");

    let def_input = pa.default_input_device().expect("Unable to get default device");
    let input_info = pa.device_info(def_input).expect("Unable to get device info");
    // println!("Default input device name: {}", input_info.name);

    let latency = input_info.default_low_input_latency;
    // Set parameters for the stream settings.
    // We pass which mic should be used, how many channels are used,
    // whether all the values of all the channels should be passed in a
    // single audiobuffer and the latency that should be considered
    let input_params = StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
    // Settings for an inputstream.
    // Here we pass the stream parameters we set before,
    // the sample rate of the mic and the amount values we want
    let settings = InputStreamSettings::new(input_params, SAMPLE_RATE, buffer_size as u32);

    let mut buffers = Vec::with_capacity(num_buffers);

    for _ in 0..num_buffers {
        buffers.push(Mutex::new(AudioBuffer {
            rendered: true,
            //why is this buffer_size + 3?
            // analytic: vec![Vec4 {vec: [0.0, 0.0, 0.0, 0.0]}; buffer_size + 3],
            analytic: vec![Vec4 {vec: [0.0, 0.0, 0.0, 0.0]}; fft_size],
        }));
    }
    //This creates a thread safe reference counting pointer to buffers
    let buffers = Arc::new(buffers);

    // This is a lambda which I want called with the samples
    let (receiver, callback) = {
        let mut buffer_index = 0;
        let (sender, receiver) = mpsc::channel();
        let gain = 1.0;//config.audio.gain;
        let buffers = buffers.clone();
        let mut analytic_buffer = vec![Vec4 {vec: [0.0, 0.0, 0.0, 0.0]}; buffer_size + 3];

        let mut time_index = 0;
        let mut time_ring_buffer = vec![Complex::new(0.0, 0.0); 2 * fft_size];
        // this gets multiplied to convolve stuff
        let mut complex_freq_buffer = vec![Complex::new(0.0f32, 0.0); fft_size];
        let mut complex_analytic_buffer = vec![Complex::new(0.0f32, 0.0); fft_size];
        let mut audio_sample : Sample::<'_,Complex<f32>,Complex<f32>>;

        let use_analytic_filt = false;
        let mut analytic_size = fft_size;
        let mut analytic : Vec<Complex<f32>> = Vec::with_capacity(analytic_size);

        if use_analytic_filt {
            let mut n = fft_size - buffer_size;
            if n % 2 == 0 {
                n -= 1;
            }
            analytic.clear();
            analytic = make_analytic(n, fft_size);
            analytic_size = buffer_size + 3;
        }
        let mut analytic_buffer = vec![Vec4 {vec: [0.0, 0.0, 0.0, 0.0]}; analytic_size];
        let mut fft_planner = FFTplanner::new(false);
        let fft = fft_planner.plan_fft(fft_size);
        // let mut ifft_planner = FFTplanner::new(true);
        // let ifft = ifft_planner.plan_fft(fft_size);

        let mut prev_input = Complex::new(0.0, 0.0); // sample n-1
        let mut prev_diff = Complex::new(0.0, 0.0); // sample n-1 - sample n-2
        let mut angle_lp = get_lowpass(cutoff, q);
        let mut noise_lp = get_lowpass(0.05, 0.7);
        /*
        pub struct InputCallbackArgs<'a, I: 'a> {
            pub buffer: &'a [I],
            pub frames: usize,
            pub flags: CallbackFlags,
            pub time: InputCallbackTimeInfo,
        }
        */

        (receiver, move |InputStreamCallbackArgs { buffer: data, .. }| {
            callback_function();
        })
    };

    // Registers the callback with PortAudio
    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    Ok((stream, buffers))
}


// angle between two complex numbers
// scales into [0, 0.5]
fn get_angle(v: Complex<f32>, u: Complex<f32>) -> f32 {
    // 2 atan2(len(len(v)*u − len(u)*v), len(len(v)*u + len(u)*v))
    let len_v_mul_u = v.norm_sqr().sqrt() * u;
    let len_u_mul_v = u.norm_sqr().sqrt() * v;
    let left = (len_v_mul_u - len_u_mul_v).norm_sqr().sqrt(); // this is positive
    let right = (len_v_mul_u + len_u_mul_v).norm_sqr().sqrt(); // this is positive
    left.atan2(right) / ::std::f32::consts::PI
}

// returns biquad lowpass filter
fn get_lowpass(n: f32, q: f32) -> Box<FnMut(f32) -> f32> {
    let k = (0.5 * n * ::std::f32::consts::PI).tan();
    let norm = 1.0 / (1.0 + k / q + k * k);
    let a0 = k * k * norm;
    let a1 = 2.0 * a0;
    let a2 = a0;
    let b1 = 2.0 * (k * k - 1.0) * norm;
    let b2 = (1.0 - k / q + k * k) * norm;

    let mut w1 = 0.0;
    let mut w2 = 0.0;
    // \ y[n]=b_{0}w[n]+b_{1}w[n-1]+b_{2}w[n-2],
    // where
    // w[n]=x[n]-a_{1}w[n-1]-a_{2}w[n-2].
    Box::new(move |x| {
        let w0 = x - b1 * w1 - b2 * w2;
        let y = a0 * w0 + a1 * w1 + a2 * w2;
        w2 = w1;
        w1 = w0;
        y
    })
}

// FIR analytical signal transform of length n with zero padding to be length m
// real part removes DC and nyquist, imaginary part phase shifts by 90
// should act as bandpass (remove all negative frequencies + DC & nyquist)
fn make_analytic(n: usize, m: usize) -> Vec<Complex<f32>> {
    use ::std::f32::consts::PI;
    assert_eq!(n % 2, 1, "n should be odd");
    assert!(n <= m, "n should be less than or equal to m");
    // let a = 2.0 / n as f32;
    let mut fft_planner = FFTplanner::new(false);
    //this probably doesn't need to be mut
    let mut fft = fft_planner.plan_fft(m);

    let mut impulse = vec![Complex::new(0.0, 0.0); m];
    let mut freqs = impulse.clone();

    let mid = (n - 1) / 2;

    impulse[mid].re = 1.0;
    let re = -1.0 / (mid - 1) as f32;
    for i in 1..mid+1 {
        if i % 2 == 0 {
            impulse[mid + i].re = re;
            impulse[mid - i].re = re;
        } else {
            let im = 2.0 / PI / i as f32;
            impulse[mid + i].im = im;
            impulse[mid - i].im = -im;
        }
        // hamming window
        let k = 0.53836 + 0.46164 * (i as f32 * PI / (mid + 1) as f32).cos();
        impulse[mid + i] = impulse[mid + i].scale(k);
        impulse[mid - i] = impulse[mid - i].scale(k);
    }
    fft.process(&mut impulse, &mut freqs);
    freqs
}

#[test]
fn test_analytic() {
    let m = 1024; // ~ 40hz
    let n = m / 4 * 3 - 1; // overlap 75%
    let freqs = make_analytic(n, m);
    // DC is below -6db
    assert!(10.0 * freqs[0].norm_sqr().log(10.0) < -6.0);
    // 40hz is above 0db
    assert!(10.0 * freqs[1].norm_sqr().log(10.0) > 0.0);
    // -40hz is below -12db
    assert!(10.0 * freqs[m-1].norm_sqr().log(10.0) < -12.0);
    // actually these magnitudes are halved bc passband is +6db
}

#[test]
fn test_lowpass() {
    let mut lp = get_lowpass(0.5, 0.71);
    println!("{}", lp(1.0));
    for _ in 0..10 {
        println!("{}", lp(0.0));
    }
    for _ in 0..10 {
        assert!(lp(0.0).abs() < 0.5); // if it's unstable, it'll be huge
    }
}
