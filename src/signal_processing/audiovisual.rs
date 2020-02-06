use std::{thread, time};
//takes a vector representing the amplitude of sampled frequencies
//determines a colour representing the sample
fn audio_to_weighted_colour(spectrum_sample : &Vec<f32>) -> std::io::Result<([u8;3])> {
   let ratio = 3;
   let colour_byte = 255.0f32;
   let split_to : usize = spectrum_sample.len()/ratio as usize;
   let mut weights = [0.0f32;3];
   //determines the sum of the amplitude of all sampled frequencies
   let mut sub_iter = 0;
   for (idx, iter) in spectrum_sample.iter().enumerate() {
       if idx == split_to || idx == split_to*(ratio-1) {
           sub_iter += 1;
       }
       if (weights[sub_iter] + iter.abs()).is_nan() {
            weights[sub_iter] += 0.0f32;
       }
       else {
            weights[sub_iter] += iter.abs();
       }
   }
   let weight_sum : f32 = weights.iter().sum();
   let average_sum = weight_sum/weights.len() as f32;
   let colour_ratio = colour_byte/ratio as f32;
   //for each third of the amplitude determine 
   let mut byte_weights = [0u8;3];
   for (idx, new_weight) in weights.iter_mut().enumerate() {
    //could also be done as 
   // for new_weight in &mut weights {
       let weight_ratio = *new_weight/average_sum;
       *new_weight = weight_ratio * colour_ratio;
       byte_weights[idx] = *new_weight as u8;
   }

   Ok(byte_weights) 
}


/*
    From Wavelength to RGB in Python - https://www.noah.org/wiki/Wavelength_to_RGB_in_Python
    == A few notes about color ==

    Color   Wavelength(nm) Frequency(THz)
    red     620-750        484-400
    Orange  590-620        508-484
    Yellow  570-590        526-508
    green   495-570        606-526
    blue    450-495        668-606
    Violet  380-450        789-668

    f is frequency (cycles per second)
    l (lambda) is wavelength (meters per cycle)
    e is energy (Joules)
    h (Plank's constant) = 6.6260695729 x 10^-34 Joule*seconds
                         = 6.6260695729 x 10^-34 m^2*kg/seconds
    c = 299792458 meters per second
    f = c/l
    l = c/f
    e = h*f
    e = c*h/l

    List of peak frequency responses for each type of 
    photoreceptor cell in the human eye:
        S cone: 437 nm
        M cone: 533 nm
        L cone: 564 nm
        rod:    550 nm in bright daylight, 498 nm when dark adapted. 
                Rods adapt to low light conditions by becoming more sensitive.
                Peak frequency response shifts to 498 nm.
*/

const MINIMUM_VISIBLE_WAVELENGTH :u16 = 380;
const MAXIMUM_VISIBLE_WAVELENGTH :u16 = 740;

fn wavelength_to_rgb(wavelength : f32, gamma : f32) -> std::io::Result<([u8;3])> {
    let red : f32;
    let green : f32;
    let blue : f32;
    thread::sleep(time::Duration::from_secs(10));

    if wavelength > 440.0 && wavelength < 490.0 {
        let attenuation = 0.3 + 0.7*(wavelength 
            - MINIMUM_VISIBLE_WAVELENGTH as f32);
        red = (-wavelength - 440.0) / (440.0 - MINIMUM_VISIBLE_WAVELENGTH as f32)
             * attenuation * gamma;
        green = 0.0;
        blue = 1.0 * attenuation;
    }
    else if wavelength >= 440.0 && wavelength <= 490.0 {
        red = 0.0;
        green = ((wavelength - 440.0) / (490.0 - 440.0)) * gamma;
        blue = 1.0;
    }
    else if wavelength >= 490.0 && wavelength <= 510.0 {
        red = 0.0;
        green = 1.0;
        blue = (-(wavelength - 510.0) / (510.0 - 490.0)) * gamma;
    }
    else if wavelength >= 510.0 && wavelength <= 580.0 {
        red = ((wavelength - 510.0) / (580.0 - 510.0)) * gamma;
        green = 1.0;
        blue = 0.0;
    }
    else if wavelength >= 580.0 && wavelength <= 645.0 {
        red = 1.0;
        green = (-(wavelength - 645.0) / (645.0 - 580.0)) * gamma;
        blue = 0.0;
    }
    else if wavelength >= 645.0 && wavelength 
        <= MAXIMUM_VISIBLE_WAVELENGTH as f32 {
        let attenuation = 0.3 + 0.7 * 
            (MAXIMUM_VISIBLE_WAVELENGTH as f32 - wavelength) 
            / (MAXIMUM_VISIBLE_WAVELENGTH as f32 - 645.0);
        red = (1.0 * attenuation) * gamma;
        green = 0.0;
        blue = 0.0;
    }
    else {
        red = 0.0;
        green = 0.0;
        blue = 0.0;
    }
    let rgb = [(255.0 * red) as u8, (255.0 * green) as u8, (255.0 * blue) as u8];

    Ok(rgb)
}

fn map_synthesia(audio_range : [f32; 2], audio_value : f32) -> std::io::Result<(f32)> {
//affline transform
//for now audio_range[0] is min and audio_range[1] is max
    let res = (audio_value - audio_range[0]) 
        * ((MAXIMUM_VISIBLE_WAVELENGTH-MINIMUM_VISIBLE_WAVELENGTH) as f32)
        /(audio_range[1] - audio_range[0]) + MINIMUM_VISIBLE_WAVELENGTH as f32;
    Ok(res)
}