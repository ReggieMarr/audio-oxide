/*
This module contains components which are used for processing data.
It contains two main structs, Sample and Scope.

It also republishes the module audiovisual. Used to project audio data, visually.

*/
pub mod audiovisual;

use std::error::Error;
const FFT_SIZE              : usize = 1024;
const NUM_TRANSFORM_OPTIONS : usize = 3;

//What we actually want to do here is iterate through transform, filter, inverse, ect..FFT_SIZE
//and for each trait that has been implemented (on our target struct)
//we execute them and handle their output accordingly
pub trait TransformOptions<SourceType> {
    type TransformBaseType;
    //fn transform(&self, &mut input : [SourceType; FFT_SIZE])->[SourceType; FFT_SIZE] {
    //    // default implementation does nothing
    //}
    ////filter may have to have coeffcient as arg
    //fn filter(&self, &mut input : [SourceType; FFT_SIZE])->[SourceType; FFT_SIZE] {
    //    // default implementation does nothing
    //}
    //fn inverse_transform(&self, &mut input : [SourceType; FFT_SIZE])->[SourceType; FFT_SIZE] {
    //    // default implementation does nothing
    //}
    //utimately we need some way to check if any of these have been implemented in
    //something like the cycle_through function
    //rather than options maybe have identifiers?
    //fn cycle_transforms(&self, function_vec : Vec<Box<dyn Fn(&mut [SourceType; FFT_SIZE])>>) {
    fn process(&self, function_vec : Vec<Box<dyn Fn(&mut SourceType)>>) {
           //default do nothing, maybe change this?
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
/*
This is a struct which represents a sample.
The sample is created by providing some series of data points.
Optionally we may provide the scope by which these data points fall under.
If the scope is not provided it is assumed the scope is defined as [0;sizeof(sample)]
If some transform function is provided then the
*/
pub struct Sample<'a, SampleType, ReturnType> {
    //pub data_points : &'a [SampleType; FFT_SIZE],
    pub data_points : &'a mut SampleType,
    pub scope : Scope,
    pub output_data : Option<ReturnType>,
    /*
    Might want to have a "mapped data type"
    which maps the data uniformly to an array the
    size of the scope
    */
}

//this is an interesting idea but lets be more straight forward for now
//impl<SourceType> TransformOptionsTrait<SourceType> for Sample<'_, SourceType, SourceType> {
//    type TransformBaseType = SourceType;
//    //fn cycle_transforms(&self, function_vec : Vec<Box<dyn Fn(&mut [SourceType; FFT_SIZE])>>) {
//    fn cycle_transforms(&self, function_vec : Vec<Box<dyn Fn(&mut SourceType)>>) {
//        let mut tmp_input = *self.data_points;
//        for func in function_vec {
//            func(& mut tmp_input)
//        }
//        self.output_data = Some(tmp_input)
//    }
//}

impl<'a, T, R> Sample::<'a, T, R>
    where Sample<'a, T, R> : TransformOptions<T>,
          &'a T : IntoIterator,
          <& 'a T as IntoIterator>::IntoIter : ::std::iter::ExactSizeIterator
{
    fn new(input_data : T, input_scope : Option<Scope>)->Self{
        let cfg_scope : Scope;
        if let Some(_) = input_scope {
            cfg_scope = input_scope.unwrap();
        }
        else {
            cfg_scope = Scope::new(0, FFT_SIZE);
        }

        let cfg_output = None;//self.cycle_transforms();
        Sample {
            data_points : &input_data,
            scope : cfg_scope,
            //we only need an output if we have multiple
            //types otherwise just mutate the input
            output_data : cfg_output
        }
    }
}

/*impl<T, R> Sample::<'_, T, R> {
 *    //I feel like this could be implemented using traits on TransformOptions
 *    //pub fn new(input_data : [T; FFT_SIZE], input_scope : Option<Scope>, transform_opt : TransformOptions<R>)->Self{
 *    pub fn new(input_data : [T; FFT_SIZE], input_scope : Option<Scope>)->Self{
 *        //let cfg_output = transform_opt.cycle_through(input_data);
 *        let cfg_scope : Scope;
 *        if let Some(_) = input_scope {
 *            cfg_scope = input_scope.unwrap();
 *        }
 *        else {
 *            cfg_scope = Scope::new(0, FFT_SIZE);
 *        }
 *
 *        Sample {
 *            data_points : &input_data,
 *            scope : cfg_scope,
 *            //we only need an output if we have multiple
 *            //types otherwise just mutate the input
 *            output_data : cfg_output
 *        }
 *    }
 *
 *    //Consider using a "RollingSample" for this functionality
 *    // fn cycle(&self)->std::io::Result<()> {
 *    //     if self.output_data.is_none() {
 *    //         panic!("Output data is none!");
 *    //     }
 *    //     self.data_points = self.output_data.unwrap();
 *    // }
 *    //We can update our sample with new data, or a new data_transform function
 *    //TODO: May want to add some way to update the scope and/or break this up
 *    //TODO:May want to introduce some lifetime members here
 *    pub fn update(&self, new_data : Option<Vec<T>>, new_transform : Option<TransformOptions<T>>) -> std::io::Result<()> {
 *        if new_data.is_none() && new_transform.is_none() {
 *            panic!("Update cannot be called with no new data or new transform!");
 *        }
 *        else if !new_data.is_none() && !new_transform.is_none() {
 *            unimplemented!();
 *        }
 *        else if new_data.is_none() && !new_transform.is_none() {
 *            let transform_func = new_transform.unwrap();
 *            //What if we've changed type ? we should check for that if we cant handle it
 *            //self.output_data = Some(transform_func(self.data_points));
 *        }
 *        else if !new_data.is_none() && new_transform.is_none() {
 *            let data = new_data.unwrap();
 *            if self.data_points.len() != data.len() {
 *                panic!("Sample can only be updated with data that shares scope");
 *            }
 *            self.data_points = &data;
 *            if let Some(_) = self.output_data {
 *                //let new_output = transform_opt.cycle_through(input_data);
 *            }
 *        }
 *        else {
 *            panic!("Should not have gotten here")
 *        }
 *        Ok(())
 *    }
 *}*/
