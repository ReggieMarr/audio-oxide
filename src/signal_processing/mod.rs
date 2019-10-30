use num::complex::Complex;
/*
This module contains components which are used for processing data.
It contains two main structs, Sample and Scope.

It also republishes the module audiovisual. Used to project audio data, visually.

*/
pub mod audiovisual;
/*
This is a struct which represents a sample.
The sample is created by providing some series of data points.
Optionally we may provide the scope by which these data points fall under.
If the scope is not provided it is assumed the scope is defined as [0;sizeof(sample)]
If some transform function is provided then the
*/
pub struct Sample<'a, SampleType,TransformType> {
    data_points : &'a Vec<SampleType>,
    scope : Scope,
    output_data : Option<TransformType>,
    /*
    Might want to have a "mapped data type"
    which maps the data uniformly to an array the
    size of the scope
    */
}

enum UpdateType {
    WithNewSample,
    WithNewFunction
}

impl<T,R : Default> Sample::<'_, T, R> {
    //DANGER! No idea what &'static does, it may make it difficult to implement the update funciton
    fn new(input_data : &'static Vec<T>, input_scope : Scope, data_transform : Option<fn(&Vec<T>,&R)>)
        ->Sample<T,R>{
        let mut output : Option<R> = None;
        if let Some(_gen_func) = data_transform {
            let transform_func = data_transform.unwrap();
            let output_ref :&R;
            transform_func(&input_data, output_ref);
            output = Some(*output_ref);
        }
        Sample {
            data_points : &input_data,
            scope : input_scope,
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
    fn update(&self, new_data : Option<Vec<T>>, new_transform : Option<fn(&Vec<T>)->R>) -> std::io::Result<()> {
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


struct AnalyzedDataPoint {
    //a single point on a complex unit circle
    complex_point : Complex<f32>,
    //the frequency of the point of the unit circle
    sample_freq : f32,
    //average angular noise
    angular_noise : f32
    //optionally we could also add angular velocity here
}

pub struct AnalyzedSample {
    data_point : Vec<AnalyzedDataPoint>
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
