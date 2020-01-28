//This should probably live a level up
use rustfft::{FFTplanner, FFT};
use std::sync::{Arc, Mutex};
use crate::data_stream::audio_stream as local_mod;

use num::complex::Complex;
//defines (should get moved somewhere else)
use local_mod::common::{
    FFT_SIZE,
    GAIN,
    BUFF_SIZE,
    ANGLE_Q,
    NOISE_Q,
    ANGLE_CUTOFF,
    NOISE_CUTOFF,
    CHANNELS,
    NUM_BUFFERS,
    SAMPLE_RATE,
};

use local_mod::common::{
    AudioSample,
    AudioStream,
    New,
    Package,
};

use crate::signal_processing::{Sample, TransformOptions};

//impl<'stream_life, DataStreamType> New for AudioStream<'stream_life, DataStreamType> {
//
//}
impl<'stream_life, ADC, BufferT> AudioStream<'stream_life, ADC, BufferT> {
    pub fn peak_current_sample(&self)->Mutex<Sample<'stream_life, ADC, BufferT>> {
        self.buffer[self.current_buff][self.current_sample]
    }
}

impl<'stream_life, ADC, BufferT> Package<'stream_life, ADC, BufferT> for AudioStream<'stream_life, ADC, BufferT>
    where AudioStream<'stream_life, ADC, BufferT> : IntoIterator,
    //where ADC : IntoIterator,
{
    fn package<Bool>(&self, package_item : AudioStream<ADC, BufferT>)->std::io::Result<Bool> {
        static mut analytic_buffer : Vec<AudioSample> = Vec::with_capacity(BUFF_SIZE + 3);//vec![AudioSample::default(); self.buffer_size + 3];
        //this should be stuck to the type used in self
        static mut prev_input : Complex<f32> = Complex::new(0.0, 0.0);
        static mut prev_diff : Complex<f32> = Complex::new(0.0, 0.0);
        //These are both config values
        let angle_lp = get_lowpass(ANGLE_CUTOFF, ANGLE_Q);
        let noise_lp = get_lowpass(NOISE_CUTOFF, NOISE_Q);
        //we need to understand this better, why should we only do this if the filter is applied?
        //if let Some(_) = self.filter {
            analytic_buffer[0] = analytic_buffer[BUFF_SIZE];
            analytic_buffer[1] = analytic_buffer[BUFF_SIZE + 1];
            analytic_buffer[2] = analytic_buffer[BUFF_SIZE + 2];
        //}
        //this is real bad to do
        let freq_res = SAMPLE_RATE as f32 / FFT_SIZE as f32;
        // for (&x, y) in complex_analytic_buffer[fft_size - buffer_size..].iter().zip(analytic_buffer[3..].iter_mut()) {
        //this takes 256 points from the complex_freq_buffer into the analytic_buffer
        // for (&x, y) in complex_freq_buffer[(fft_size - buffer_size)..].iter().zip(analytic_buffer[3..].iter_mut()) {
        let freq_iter = package_item.iter().zip(analytic_buffer.iter_mut());
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
                angular_noise : noise_lp((angle - output).abs()), // average angular noise
                angular_velocity : output.exp2(),
            }
        }

        static mut buffer_index : usize = 0;
        //what is rendered and why would dropped represent its inverse ?
        let dropped = {
            let mut buffer = self.buffer[buffer_index].lock().unwrap();
            let rendered = buffer.rendered;
            buffer.analytic.copy_from_slice(&analytic_buffer);
            buffer.rendered = false;
            !rendered
        };
        buffer_index = (buffer_index + 1) % NUM_BUFFERS;
        Ok(dropped)
    }

}


//impl<'stream_life> AudioStream<'stream_life, Vec<f32>>
//{
//    //this could be a trait for audio stuff like make_mono
//    fn normalize_sample(&self, data : Vec<f32>, time_index : usize)->std::io::Result<[Complex<f32>; FFT_SIZE]> {
//        //should use const but for now we will hard code FFT_SIZE
//        static mut sample_buffer : [Complex<f32>; FFT_SIZE] = arr![Complex::new(0.0, 0.0); 1024];
//        //should assert that split point is indeed the middle of the buffer
//        let (left, right) = sample_buffer.split_at_mut(FFT_SIZE);
//        //This takes the buffer input to the stream and then begins describing the
//        //input using complex values on a unit circle.alloc
//        let data = data as Vec<f32>;
//        for ((x, t0), t1) in data.chunks(CHANNELS)
//            .zip(left[time_index..(time_index + BUFF_SIZE)].iter_mut())
//            .zip(right[time_index..(time_index + BUFF_SIZE)].iter_mut())
//        {
//            let mono = Complex::new(GAIN * (x[0] + x[1]) / 2.0, 0.0);
//            *t0 = mono;
//            *t1 = mono;
//        }
//        Ok(sample_buffer)
//    }
//    //May want to encapsulate some of the arguments here. Additionally we are breaking the function does one thing rule
//    //We actually update the time index and the sample buffer
//    //this stuff may actually make more sense to be implemented on the audio side
//    //maybe even as a trait like cast_sample
//    fn clean_stream(&self, data : Vec<f32>)->std::io::Result<[Complex<f32>; FFT_SIZE]> {
//        //this updates the time index as we continue to sample the audio stream
//        static mut time_index : usize = 0;
//        let normalized_sample = self.normalize_sample(data, time_index)?;
//        time_index = ((time_index + BUFF_SIZE) % FFT_SIZE).try_into().unwrap();
//        Ok(normalized_sample)
//    }
//    //fn coalece(&self, input_adc : DataStreamType)->std::io::Result<bool> {
//    fn coalece(&self, input_adc : Vec<f32>)->std::io::Result<(AudioSample)> {
//        self.clean_stream(input_adc);
//        self.thalweg.update(Some(input_adc), None)?;
//        Ok(self.thalweg.output_data.unwrap())
//    }
//}


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
fn get_lowpass(n: f32, q: f32) -> Box<dyn FnMut(f32) -> f32> {
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
//fn make_analytic(n: usize, m: usize) -> Vec<Complex<f32>> {
fn make_analytic(n: usize) -> [Complex<f32>; FFT_SIZE] {
    use ::std::f32::consts::PI;
    assert_eq!(n % 2, 1, "n should be odd");
    assert!(n <= FFT_SIZE, "n should be less than or equal to FFT_SIZE");
    // let a = 2.0 / n as f32;
    let mut fft_planner = FFTplanner::new(false);
    //this probably doesn't need to be mut
    let mut fft = fft_planner.plan_fft(FFT_SIZE);

    //let mut impulse = vec![Complex::new(0.0, 0.0); m];
    let mut impulse = [Complex::new(0.0, 0.0); FFT_SIZE];
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
