/*
This module contains components which are used for processing data.
It contains two main structs, Sample and Scope.

It also republishes the module audiovisual. Used to project audio data, visually.

*/
pub mod audiovisual;

const FFT_SIZE              : usize = 1024;
const NUM_TRANSFORM_OPTIONS : usize = 3;

//What we actually want to do here is iterate through transform, filter, inverse, ect..FFT_SIZE
//and for each trait that has been implemented (on our target struct)
//we execute them and handle their output accordingly
pub trait TransformOptionsTrait<SourceType> {
    type TransformBaseType;
    fn transform(&self, &mut input : [SourceType; FFT_SIZE])->[SourceType; FFT_SIZE] {
        // default implementation does nothing
    }
    //filter may have to have coeffcient as arg
    fn filter(&self, &mut input : [SourceType; FFT_SIZE])->[SourceType; FFT_SIZE] {
        // default implementation does nothing
    }
    fn inverse_transform(&self, &mut input : [SourceType; FFT_SIZE])->[SourceType; FFT_SIZE] {
        // default implementation does nothing
    }
    //utimately we need some way to check if any of these have been implemented in
    //something like the cycle_through function
}

pub struct TransformOptions<SourceType>
    where SourceType : std::ops::MulAssign
{
    transform         : Option<Box<dyn Fn(&mut [SourceType; FFT_SIZE], &mut [SourceType; FFT_SIZE])>>,
    filter            : Option<[SourceType; FFT_SIZE]>,
    inverse_transform : Option<Box<dyn Fn(&mut [SourceType; FFT_SIZE], &mut [SourceType; FFT_SIZE])>>
    //should try anddo this in an array
    //options           : [Option; NUM_TRANSFORM_OPTIONS],
}

impl<SourceType> TransformOptions<SourceType>
    where SourceType : std::ops::MulAssign
{
    pub fn cycle_through(&self, mut input : [SourceType; FFT_SIZE])->[SourceType; FFT_SIZE] {
        //let input : [T ; FFT_SIZE] = arr![T; FFT_SIZE];
        //This represents the amplitude of the signal represented as the distance from the origin on a unit circle
        //Here we transform the signal from the time domain to the frequency domain.
        //Note that humans can only hear sound with a frequency between 20Hz and 20_000Hz
        // fft.process(&mut time_ring_buffer[time_index..time_index + fft_size], &mut complex_freq_buffer[..]);
        if  let Some(_) = self.transform{
            let transform_func = self.transform.unwrap();
            let output = input;
            transform_func(&input, &output);
            input = output;
        }
        //the analytic array acts as a filter, removing the negative and dc portions
        //of the signal as well as filtering out the nyquist portion of the signal
        //Also applies the hamming window here

        // By applying the inverse fourier transform we transform the signal from the frequency domain back into the
        if  let Some(_) = self.filter {
            let filter_coefficient = self.filter.unwrap();
            /*
               this is roughly how it should go down
               | input, coefficient | {
               for input_idx in index.ter() {
                    input_idx = input_idx * coeffcient[input_idx.index];
               }
               }
            */
            //input = filter_func(&input);
            for input_idx in input.iter_mut() {
                input_idx *= filter_coefficient;
            }
        }
        // By applying the inverse fourier transform we transform the signal from the frequency domain back into the
        // time domain. However now this signal can be represented as a series of points on a unit circle.
        // ifft.process(&mut complex_freq_buffer[..], &mut complex_analytic_buffer[..]);
        if  let Some(_) = self.inverse_transform {
            let transform_func = self.inverse_transform.unwrap();
            let output = input;
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
pub struct Sample<'a, SampleType, ReturnType> {
    pub data_points : &'a [SampleType; FFT_SIZE],
    pub scope : Scope,
    pub output_data : Option<ReturnType>,
    /*
    Might want to have a "mapped data type"
    which maps the data uniformly to an array the
    size of the scope
    */
}



impl<T, R> Sample::<'_, T, R> {
    //I feel like this could be implemented using traits on TransformOptions
    pub fn new(input_data : [T; FFT_SIZE], input_scope : Option<Scope>, transform_opt : TransformOptions<R>)->Self{
        let cfg_output = transform_opt.cycle_through(input_data);
        let cfg_scope : Scope;
        if let Some(_) = input_scope {
            cfg_scope = input_scope.unwrap();
        }
        else {
            cfg_scope = Scope::new(0, FFT_SIZE);
        }

        Sample {
            data_points : &input_data,
            scope : cfg_scope,
            //we only need an output if we have multiple
            //types otherwise just mutate the input
            output_data : cfg_output
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
    pub fn update(&self, new_data : Option<Vec<T>>, new_transform : Option<TransformOptions<T>>) -> std::io::Result<()> {
        if new_data.is_none() && new_transform.is_none() {
            panic!("Update cannot be called with no new data or new transform!");
        }
        else if !new_data.is_none() && !new_transform.is_none() {
            unimplemented!();
        }
        else if new_data.is_none() && !new_transform.is_none() {
            let transform_func = new_transform.unwrap();
            //What if we've changed type ? we should check for that if we cant handle it
            //self.output_data = Some(transform_func(self.data_points));
        }
        else if !new_data.is_none() && new_transform.is_none() {
            let data = new_data.unwrap();
            if self.data_points.len() != data.len() {
                panic!("Sample can only be updated with data that shares scope");
            }
            self.data_points = &data;
            if let Some(_) = self.output_data {
                //let new_output = transform_opt.cycle_through(input_data);
            }
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
    fn new(init_start : usize, init_end : usize)->Self {
        assert!(init_start < init_end);
        Scope {
            start : init_start,
            end : init_end,
            size : init_end - init_start
        }
    }
}
