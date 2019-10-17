
extern crate bincode;
use serde::ser::{Serialize, SerializeStruct, Serializer};//, Deserialize, Deserializer};
use std::os::raw::c_char;
use std::slice;

//might not need this
#[repr(C)]
struct Buffer {
    data : *mut u8,
    len : usize
}

struct Coordinate {
    pub x : u8,
    pub y : u8,
    pub z : u8
}

type PixelColour = [u8;3];
// #[derive(Deserialize, Serialize)]
struct Pixel {
    //Named with a U because 🇨🇦
    pub colour : PixelColour,
    pub on_status : bool,
    /* 
    We could transition to this, using only the dimensions which are relevant
    pub coordinate : Coordinate
    */
    pub index : u8
}

impl Pixel {
    fn new(setup_colour : Option<PixelColour>, setup_index : Option<u8>) -> Pixel {
        let mut prologue_colour = [0u8;3];
        if let Some(x) = setup_colour {
            prologue_colour = setup_colour.unwrap();
        }
        let mut prologue_index = 0u8;
        if let Some(x) = setup_index {
            prologue_index = setup_index.unwrap();
        }
        Pixel { 
            colour : prologue_colour,
            on_status : true,
            index : prologue_index
        }
    }
}

//This is probably not required anymore
impl serde::ser::Serialize for Pixel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Pixel", 2)?;
        state.serialize_field("colour", &self.colour)?;
        state.serialize_field("index", &self.index)?;
        state.end()
    }
}


struct Scope {
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

struct Sample<'a, SampleType,TransformType> {
    data : &'a Vec<SampleType>,
    scope : Scope,
    output_data : Option<TransformType>,
    /*
    Might want to have a "mapped data type"
    which maps the data uniformly to an array the
    size of the scope
    */
}

impl<T,R : Default> Sample<'_, T, R> {
    //DANGER! No idea what &'static does, it may make it difficult to implement the update funciton
    fn new(input_data : &'static Vec<T>, input_scope : Scope, data_transform : Option<fn(&Vec<T>)->R>)
        ->Sample<T,R>{
        let mut output : Option<R> = None;
        if let Some(gen_func) = data_transform {
            let transform_func = data_transform.unwrap();
            output = Some(transform_func(&input_data));
        }
        Sample {
            data : &input_data,
            scope : input_scope,
            output_data : output
        }
    }
}

const DEFAULT_MESSAGE_SIZE : usize = 1024;
//each led colour is represented by the value of 3 bytes (r,g,b)
const COLOUR_SIZE : usize = 256*3;

struct PixelStrip {
    strip : Vec<Pixel>
}

//this should be called from multiple threads
impl PixelStrip {
    //need an update here that does essentially the same thing except doesnt create a new instance
    //this error probably has something to do with PixelUsing generic types
    fn new(scope : Scope, colours : Vec<PixelColour>)->std::io::Result<(PixelStrip)> {
        //suboptimal but the best we can do for now
        assert!(scope.size >= colours.len());
        let mut pixel_strip : Vec<Pixel> = Vec::with_capacity(scope.end - scope.start);
        for (idx, _) in (scope.start..scope.end).enumerate() {
            pixel_strip.push(Pixel::new(Some(colours[idx]), Some(idx as u8)));
        }
        Ok(PixelStrip {
            strip : pixel_strip
        })
    }
}

trait PixelStripSerialize {
    fn get_packet(&self) -> std::io::Result<(Box<([u8;1024])>)>;
}

impl PixelStripSerialize for PixelStrip {

    fn get_packet(&self) -> std::io::Result<(Box<([u8;1024])>)> {
        
        let pixel_strip = &self.strip;
        // assert(pixel_strip.len(), led_num);
        //the message_size is determined by the number of leds multiplied by the memory required for the colour
        // let message_size : usize = led_num*COLOUR_SIZE;
        
        // let mut packet_byte_array = vec![0 as u8; message_size];
        let mut packet_byte_array = [0 as u8; 1024];
        // for (idx, pixel) in pixel_strip.iter().enumerate() {
        // let mut stdout = StandardStream::stdout(ColorChoice::Always);
        for pixel_idx in (0..DEFAULT_MESSAGE_SIZE).step_by(4) {
            //since we are always sending a message with an array of 256 pixels
            // packet_byte_array.push(pixel.index);
            // packet_byte_array.push(pixel_idx as u8);

            if pixel_idx < pixel_strip.len() {
                packet_byte_array[pixel_idx] = pixel_strip[pixel_idx].index;
                packet_byte_array[pixel_idx+1] = pixel_strip[pixel_idx].colour[0];
                packet_byte_array[pixel_idx+2] = pixel_strip[pixel_idx].colour[1];
                packet_byte_array[pixel_idx+3] = pixel_strip[pixel_idx].colour[2];
                // stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(
                //     packet_byte_array[pixel_idx+0],
                //     packet_byte_array[pixel_idx+1], 
                //     packet_byte_array[pixel_idx+2]))));
                // println!("▀");
            } 
            // else {
                //without this we could have an error where we 0 out the first led
            //     let pixel_real_idx = pixel_idx as u8/4u8;
            //     packet_byte_array[pixel_idx] = pixel_real_idx;
            // }
        }
        Ok(Box::new(packet_byte_array))
    }
}



// fn make_random_led_vec(strip_size : usize) -> Vec<Vec<u8>> {
//         let mut test_leds : Vec<Vec<u8>> = Vec::with_capacity(strip_size);
//         let mut rng = rand::thread_rng();

//         for _ in 0..strip_size {
//             let led_idx: Vec<u8> = (0..3).map(|_| {
//                 rng.gen_range(0,255)
//                 }).collect();
//             test_leds.push(led_idx);
//         }
//         test_leds
// }