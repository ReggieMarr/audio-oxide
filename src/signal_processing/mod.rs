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
    pub start : usize,
    pub end : usize,
    pub size : usize
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
Essentially what we are infering is whether the data_points can/should be subdivided or
if they represent the base elements of some data set
*/
//pub struct Sample<'a, SampleType> {
pub struct Sample<SampleType> {
    //pub data_points : &'a mut SampleType,
    //pub data_points : Vec<Option<SampleType>>,
    pub data_points : Option<SampleType>,
    pub scope : Scope,
    pub size : usize
}

impl<T> Sample::<T>
    //where Sample<T> : TransformOptions<T>,
    //      T : IntoIterator,
    //      <T as IntoIterator>::IntoIter : ::std::iter::ExactSizeIterator
{
    pub fn new(setup_data : Option<T>, setup_scope : Option<Scope>, setup_size : Option<usize>)->Self{
        let cfg_size : usize;
        let cfg_scope : Scope;
        let cfg_data : Option<T>;
        if let Some(_) = setup_size {
            cfg_size = setup_size.unwrap();
            if let Some(_) = setup_scope {
                cfg_scope = setup_scope.unwrap();
            }
            else {
                //Note that if we do not have a
                cfg_scope = Scope::new(0, cfg_size);
            }
        }
        else {
            //Note that if we do not have a
            if let Some(_) = setup_scope {
                cfg_scope = setup_scope.unwrap();
            }
            else {
                //Note that if we do not have a
                panic!("Cannot imply size")
            }
        }

        //if let Some(_) = setup_data {
        //    cfg_data = setup_data.unwrap();
        //}
        //else {
        //    //In the future this should be set using preproccessor commands
        //    cfg_data =
        //}

        Sample {
            data_points : setup_data,
            scope : cfg_scope,
            size : cfg_size
        }
    }
}
