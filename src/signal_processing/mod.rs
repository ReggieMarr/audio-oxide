use num::complex::Complex;
/*
This module contains components which are used for processing data.
It contains two main structs, Sample and Scope.

It also republishes the module audiovisual. Used to project audio data, visually.

*/
pub mod audiovisual;
use arr_macro::arr;

enum UpdateType {
    WithNewSample,
    WithNewFunction
}
const FFT_SIZE        : f32   = 1024;

const NUM_TRANSFORM_OPTIONS = 3
pub struct Transform_Options<SourceType> {
    transform         : Option<dyn Fn(&mut [SourceType; FFT_SIZE], &mut [SourceType; FFT_SIZE])>,
    filter            : Option<dyn Fn(&mut [SourceType; FFT_SIZE], [SourceType; FFT_SIZE])>,
    inverse_transform : Option<dyn Fn(&mut [SourceType; FFT_SIZE], &mut [SourceType; FFT_SIZE])>
    //should try anddo this in an array
    options           : [Option; NUM_TRANSFORM_OPTIONS],
}

impl<SourceType> for Transform_Options<SourceType> {
    fn cycle_through(&self, input : [SourceType; FFT_SIZE])-> output : [SourceType; FFT_SIZE] {
        //let input : [T ; FFT_SIZE] = arr![T; FFT_SIZE];
        //This represents the amplitude of the signal represented as the distance from the origin on a unit circle
        //Here we transform the signal from the time domain to the frequency domain.
        //Note that humans can only hear sound with a frequency between 20Hz and 20_000Hz
        // fft.process(&mut time_ring_buffer[time_index..time_index + fft_size], &mut complex_freq_buffer[..]);
        if  let Some(_) = self.transform_opt.transform {
            let transform_func = transform_opt.transform.unwrap();
            let output = input_data.clone();
            transform_func(&input, &output);
            input = output;
        }
        //the analytic array acts as a filter, removing the negative and dc portions
        //of the signal as well as filtering out the nyquist portion of the signal
        //Also applies the hamming window here

        // By applying the inverse fourier transform we transform the signal from the frequency domain back into the
        if  let Some(_) = self.transform_opt.filter {
            let filter_func = transform_opt.filter.unwrap();
            /*
               this is roughly how it should go down
               | input, coefficient | {
               for input_idx in index.ter() {
                    input_idx = input_idx * coeffcient[input_idx.index];
               }
               }
            */
            input = filter(&input);
        }
        // By applying the inverse fourier transform we transform the signal from the frequency domain back into the
        // time domain. However now this signal can be represented as a series of points on a unit circle.
        // ifft.process(&mut complex_freq_buffer[..], &mut complex_analytic_buffer[..]);
        if  let Some(_) = self.transform_opt.inverse_transform {
            let transform_func = transform_opt.inverse_transform.unwrap();
            let output = input_data.clone();
            transform_func(&input, &output);
            input = output;
        }
        input
    }
}

/*
This is a struct which represents a sample.
The sample is created by providing some series of data points.
Optionally we may provide the scope by which these data points fall under.
If the scope is not provided it is assumed the scope is defined as [0;sizeof(sample)]
If some transform function is provided then the
*/
pub struct Sample<'a, SampleType> {
    data_points : &'a Vec<SampleType>,
    scope : Scope,
    output_data : Option<SampleType>,
    /*
    Might want to have a "mapped data type"
    which maps the data uniformly to an array the
    size of the scope
    */
}

impl<T : Default> Sample::<'_, T> {
    //I feel like this could be implemented using traits on Transform_Options
    fn new(input_data : &'static [T; FFT_SIZE], input_scope : Scope, transform_opt : Transform_Options<T>)->Self{
        let ouput = transform_opt.cycle_through(input_data);

        Sample {
            data_points : &input_data,
            scope : input_scope,
            //we only need an output if we have multiple
            //types otherwise just mutate the input
            output_data : output
        }
    }

    //Consider using a "RollingSample" for this functionality
    // fn cycle(&self)->std::io::Result<()> {
    //     if self.output_data.is_none() {
    //         panic!("Output data is none!");
    //     }
    //     self.data_points = self.output_data.unwrap();
    // }
    //We can update our sample with new data, or a new data_transform function
    //TODO: May want to add some way to update the scope and/or break this up
    //TODO:May want to introduce some lifetime members here
    fn update(&self, new_data : Option<Vec<T>>, new_transform : Transform_Options<T>) -> std::io::Result<()> {
        if new_data.is_none() && new_transform.is_none() {
            panic!("Update cannot be called with no new data or new transform!");
        }
        else if !new_data.is_none() && !new_transform.is_none() {
            unimplemented!();
        }
        else if new_data.is_none() && !new_transform.is_none() {
            let transform_func = new_transform.unwrap();
            //What if we've changed type ? we should check for that if we cant handle it
            self.output_data = Some(transform_func(self.data_points));
        }
        else if !new_data.is_none() && new_transform.is_none() {
            let data = new_data.unwrap();
            if self.data_points.len() != data.len() {
                panic!("Sample can only be updated with data that shares scope");
            }
            self.data_points = &data;
        }
        else {
            panic!("Should not have gotten here")
        }
        Ok(())
    }
}

pub struct Scope {
    start : usize,
    end : usize,
    size : usize
}

impl Scope {
    fn new(init_start : usize, init_end : usize) -> Result<Scope,()> {
        assert!(init_start < init_end);
        Ok(Scope {
            start : init_start,
            end : init_end,
            size : init_end - init_start
        })
    }
}
