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
    Package,
    InputStreamSample,
    AudioSampleStream,
    ADCResolution
};
use crate::signal_processing::Sample;
use crate::signal_processing::Scope;

impl Package<InputStreamSample, AudioSampleStream> for AudioStream<AudioSampleStream>
{
    fn package(&mut self, proccessed_stream : InputStreamSample)
        ->std::io::Result<bool> {
        //let mut analytic_buffer : [AudioSampleStream; (BUFF_SIZE + 3)] = [AudioSampleStream::default(); (BUFF_SIZE + 3)];
        let mut analytic_buffer : Vec<AudioSampleStream> =
            vec![AudioSampleStream::default(); BUFF_SIZE + 3];
        //this should be stuck to the type used in self
        let mut prev_input : Complex<ADCResolution> = Complex::new(0.0, 0.0);
        let mut prev_diff : Complex<ADCResolution> = Complex::new(0.0, 0.0);
        //These are both config values (should be static)
        let mut angle_lp = get_lowpass(ANGLE_CUTOFF, ANGLE_Q);
        let mut noise_lp = get_lowpass(NOISE_CUTOFF, NOISE_Q);
        //we need to understand this better, why should we only do this if the filter is applied?
        //if let Some(_) = self.filter {
            analytic_buffer[0] = analytic_buffer[BUFF_SIZE];
            analytic_buffer[1] = analytic_buffer[BUFF_SIZE + 1];
            analytic_buffer[2] = analytic_buffer[BUFF_SIZE + 2];
        //}
        let mut count : f32 = 0.0;
        let freq_res = SAMPLE_RATE as ADCResolution / FFT_SIZE as ADCResolution;
        //this takes 256 points from the complex_freq_buffer into the analytic_buffer
        let freq_iter = proccessed_stream[(FFT_SIZE-BUFF_SIZE)..].iter().zip(analytic_buffer[3..].iter_mut());
        //let freq_iter = proccessed_stream.iter().zip(analytic_buffer.iter_mut());
        for (freq_idx, (&x, y)) in freq_iter.enumerate() {
            count += (x.re + x.im).exp2();
            let diff = x - prev_input; // vector
            prev_input = x;

            let angle = get_angle(diff, prev_diff).abs().log2().max(-1.0e12); // angular velocity (with processing)
            prev_diff = diff;

            let output = angle_lp(angle);

            let freq_idx = freq_res * freq_idx as ADCResolution;

            *y = AudioSample {
                complex_point : x,
                sample_freq : freq_idx,
                angular_noise : noise_lp((angle - output).abs()), // average angular noise
                angular_velocity : output.exp2(),
            }
        }
        if count < 10.0 {
            println!("Count is low! {:?}", count);
        }
        let first_freq = analytic_buffer[3].sample_freq;
        let last_freq = analytic_buffer.last().unwrap().sample_freq;
        let sample_scope = Scope::new(first_freq as usize, last_freq as usize);
        // let this_sample = Sample::new(Some(analytic_buffer),Some(sample_scope));
        // static mut current_buff : usize = 0;
        // let mut buffer_index : usize = 0;
        //this tells us whether we have a buffer that is ready for analysis
        let dropped = {
            let mut buffer = self.buffers[self.current_buff].lock().unwrap();
            let rendered = self.rendered[self.current_buff];

            buffer.data_points.copy_from_slice(&analytic_buffer[3..]);
            buffer.scope = sample_scope;
            self.rendered[self.current_buff] = false;
            !rendered
        };
        let current_buff = (self.current_buff + 1) % NUM_BUFFERS;
        self.current_buff = current_buff;
        Ok(dropped)
    }

}

// angle between two complex numbers
// scales into [0, 0.5]
fn get_angle(v: Complex<ADCResolution>, u: Complex<ADCResolution>) -> ADCResolution {
    // 2 atan2(len(len(v)*u − len(u)*v), len(len(v)*u + len(u)*v))
    let len_v_mul_u = v.norm_sqr().sqrt() * u;
    let len_u_mul_v = u.norm_sqr().sqrt() * v;
    let left = (len_v_mul_u - len_u_mul_v).norm_sqr().sqrt(); // this is positive
    let right = (len_v_mul_u + len_u_mul_v).norm_sqr().sqrt(); // this is positive
    left.atan2(right) / ::std::f32::consts::PI
}

// returns biquad lowpass filter
fn get_lowpass(n: ADCResolution, q: ADCResolution) -> Box<dyn FnMut(ADCResolution) -> ADCResolution> {
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
//fn make_analytic(n: usize, m: usize) -> Vec<Complex<ADCResolution>> {
fn make_analytic(n: usize) -> [Complex<ADCResolution>; FFT_SIZE] {
    use ::std::f32::consts::PI;
    assert_eq!(n % 2, 1, "n should be odd");
    assert!(n <= FFT_SIZE, "n should be less than or equal to FFT_SIZE");
    // let a = 2.0 / n as ADCResolution;
    let mut fft_planner = FFTplanner::new(false);
    //this probably doesn't need to be mut
    let fft = fft_planner.plan_fft(FFT_SIZE);

    //let mut impulse = vec![Complex::new(0.0, 0.0); m];
    let mut impulse = [Complex::new(0.0, 0.0); FFT_SIZE];
    let mut freqs = impulse.clone();

    let mid = (n - 1) / 2;

    impulse[mid].re = 1.0;
    let re = -1.0 / (mid - 1) as ADCResolution;
    for i in 1..mid+1 {
        if i % 2 == 0 {
            impulse[mid + i].re = re;
            impulse[mid - i].re = re;
        } else {
            let im = 2.0 / PI / i as ADCResolution;
            impulse[mid + i].im = im;
            impulse[mid - i].im = -im;
        }
        // hamming window
        let k = 0.53836 + 0.46164 * (i as ADCResolution * PI / (mid + 1) as ADCResolution).cos();
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
