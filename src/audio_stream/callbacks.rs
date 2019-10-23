use rustfft::{FFTplanner, FFT};
use num::complex::Complex;

#[derive(Copy, Clone, Debug)]
pub struct Vec4 {
    pub vec: [f32; 4],
}
implement_vertex!(Vec4, vec);

struct input_parameters {
    fft_size : usize,
    buffer_size : usize,
    use_analytic_filter : bool,
    angle_lp : Box<FnMut(f32) -> f32>,
    noise_lp : Box<FnMut(f32) -> f32>,
    fft : Option<Arc<FFT<T>>>,
    ifft : Option<Arc<FFT<T>>>
}
//break this up with more generic names
struct callback_scope_parameters {
    time_ring_buffer : Vec<Complex>
    time_index : i128,
    analytic_buffer : Vec<Vec4>

}

trait EventHandler {
    fn eventhandler(&self) {

    }
}

//come up with a better name
struct Callback {
    input : input_parameters,
    scope_param : callback_scope_parameters,
}

impl EventHandler for Callback {
    fn eventhandler(&self) {

    {
        let (left, right) = self.scope_param.time_ring_buffer.split_at_mut(fft_size);
        //This takes the buffer input to the stream and then begins describing the 
        //input using complex values on a unit circle. 
        for ((x, t0), t1) in data.chunks(CHANNELS as usize)
            .zip(left[time_index..(time_index + buffer_size)].iter_mut())
            .zip(right[time_index..(time_index + buffer_size)].iter_mut())
        {
            let mono = Complex::new(gain * (x[0] + x[1]) / 2.0, 0.0);
            *t0 = mono;
            *t1 = mono;
        }
    }
    //this updates the time index as we continue to sample the audio stream
    self.scope_param.time_index = (time_index + buffer_size as usize) % fft_size;
    //This represents the amplitude of the signal represented as the distance from the origin on a unit circle
    //Here we transform the signal from the time domain to the frequency domain. 
    //Note that humans can only hear sound with a frequency between 20Hz and 20_000Hz
    fft.process(&mut time_ring_buffer[time_index..time_index + fft_size], &mut complex_freq_buffer[..]);

    //the analytic array acts as a filter, removing the negative and dc portions
    //of the signal as well as filtering out the nyquist portion of the signal
    //Also applies the hamming window here 

    // By applying the inverse fourier transform we transform the signal from the frequency domain back into the 
    if use_analytic_filt {
        for (x, y) in analytic.iter().zip(complex_freq_buffer.iter_mut()) {
            *y = *x * *y;
        }
    }
    // By applying the inverse fourier transform we transform the signal from the frequency domain back into the 
    // time domain. However now this signal can be represented as a series of points on a unit circle.
    // ifft.process(&mut complex_freq_buffer[..], &mut complex_analytic_buffer[..]);

    if false {
        let test_freq = complex_freq_buffer.clone();
        for (freq_idx, freq) in test_freq.iter().take(fft_size/2).enumerate() {
            let bin = SAMPLE_RATE as f32 / fft_size as f32;
            // let freq_mag = f32::sqrt((freq.re as f32).exp2() + (freq.im as f32).exp2())/fft_size as f32;
            let freq_val = bin*freq_idx as f32;
            if freq_val > 200.0f32 && freq_val < 20_000.0f32 {
                println!("{:?}, {:?}", freq_val as f32, (20.0f32*((2.0f32*freq.im/fft_size as f32).abs().log10())));
            }
        }
    }
    //here we are filling the start of our array with the ned of the last one so that we have a continuous stream
    if use_analytic_filt {
        analytic_buffer[0] = analytic_buffer[buffer_size];
        analytic_buffer[1] = analytic_buffer[buffer_size + 1];
        analytic_buffer[2] = analytic_buffer[buffer_size + 2];
    }
    // time domain. However now this signal can be represented as a series of points on a unit circle.
    // ifft.process(&mut complex_freq_buffer[..], &mut complex_analytic_buffer[..]);
    let scale = fft_size as f32;
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

        let sample_freq = freq_res * freq_idx as f32;

        *y = Vec4 { vec: [
            // save the scaling for later
            x.re,
            x.im,
            sample_freq, // smoothed angular velocity
            noise_lp((angle - output).abs()), // average angular noise
        ]};
    }

    //what is rendered and why would dropped represent its inverse ?
    let dropped = {
        let mut buffer = buffers[buffer_index].lock().unwrap();
        let rendered = buffer.rendered;
        buffer.analytic.copy_from_slice(&analytic_buffer);
        buffer.rendered = false;
        !rendered
    };
    // for x in 0..num_buffers {
        // println!("noise {:?}", analytic_buffer[x]);
    // }
    buffer_index = (buffer_index + 1) % num_buffers;
    if dropped {
        // what does sender do generally ?
        sender.send(()).ok();
    }
    Continue
    }
}