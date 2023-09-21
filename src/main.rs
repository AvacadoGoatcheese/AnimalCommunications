#![allow(unused_variables, unused_imports, dead_code, unused_parens)]
use core::panic;
use std::cmp::Ordering;
use std::{io, vec};
// use std::sync::{mpsc, Mutex, Arc, MutexGuard, Condvar};
use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

/*
    What is the code supposed to do?

    Order - (1) Record --> (2) Process --> (3) Store GPS/Audio Data if process says to

    (1) Record data from microphone. Later on, deal with drivers, but now all to do is generate fake microphone data

    (2) Process - filter it (Moving average FIR) and then find the RMS value and compare to threshold value. 

    (3) Store GPS/Audio - Data will be analyzed in .wav format realistically. If to be stored, compress it to mp3 (use lossy compression)
            Throughout the algorithm, systematically store GPS coordinates (as a sub in for say, accelerometer data) and put into .txt file
            If MAX_STORAGE surpassed, maybe stop program execution?

    ---- CURRENT PROGRESS ----
    1. Currently, I am making sure a single threaded system works (with of course large slowdown of data processing at first)
    2. All data acquiration is randomized instead of actually accessing sensor values.
    3. Data is currently stored in .txt format instead of .mp3 (end goal). The implementation of .mp3 may be done from scratch 
        (as a learning experience) but will likely use a rust crate.

    ==== NEXT STEPS ====
    1. Create mp3 file for compression instead of text, and just have smaller text files.
    2. Multithreading application with properly shared data (for example, GPS data will be stored for 5 seconds if the boolean 
        for it is true, but the data storage itself will be done separately.)
 */


const WINDOW_SIZE :usize = 100;         // Size of window for normalizing, filtering, and processing to check for animal sounds
const THRESHOLD_VALUE : f32 = 0.60;     // Minimum rms value to assume animal speech occured
const KERNEL_SIZE : u8 = 5;             // Size of kernel for moving average filter within a window.
const ERROR :i32 = 1;                   // Error value (if return type is int)
const ERROR_F32 : f32 = -1.0;           // Error value (if return type is f32, aka rms)
const SUCCESS :i32 = 0;                 // Success value (if return type is int)
const SAMPLE_RATE_HZ : i32 = 44100;     // How many samples per second the microphone takes (HZ)

// Struct to contain crucial GPS data all in float format. Considering switching to strings for convenience, that is TBD
struct GpsData {
    time : f64,
    longitude : f64,
    lattitude : f64, 
    altitude : f64
}

impl Default for GpsData {
    // Default to initialize gps with bad data, will be overwritten later.
    fn default() -> GpsData {
        GpsData { time: -1.0, longitude: -1.0, lattitude: -1.0, altitude: -1.0 }
    }
}


fn main() {
    // Record
    let mut audio_buf: Vec<f32> = Vec::new(); // Contains audio data
    let mut start_time : f32 = 0.0;           // Is the start time of the current window to be analyzed

    for i in 0..10 {
        // This function will be used to access microphone data but now it is a vector of random floats
        generate_audio_data(&mut audio_buf);
    
        // Process
        audio_buf = filter_one_window(audio_buf);
        let rms : f32 = amplitude_rms(&mut audio_buf);
    
        if (rms > THRESHOLD_VALUE) {
            // STORE DATA AND DELETE OLD VALUES
            store_data(&mut audio_buf[0..(WINDOW_SIZE as usize)], &mut start_time);
            audio_buf.drain(0..(WINDOW_SIZE as usize));
        } else if (rms == ERROR_F32) {
            panic!("Vector was too small!");
        } else {
            start_time += WINDOW_SIZE as f32 / SAMPLE_RATE_HZ as f32;
        }
    }

    println!("Completed 10 samples!");
}

fn generate_audio_data(vector : &mut Vec<f32>) {
    let mut rng = rand::thread_rng();

    for _ in 0..WINDOW_SIZE {
        vector.push(rng.gen::<f32>() * 10.0);
    }
}

fn filter_one_window(mut vector : Vec<f32>) -> Vec<f32> {
    // Make sure vector is long enough
    match vector.len().cmp(&(WINDOW_SIZE as usize)) {
        Ordering::Equal => {}
        Ordering::Greater => {}
        Ordering::Less =>  { panic!("Bad vector!"); }
    }

    match (KERNEL_SIZE % 2) {
        1 => {}
        _ => {
            panic!("Please use a proper kernel size --- Kernels must be centered around the original value 
                    itself, and thus we require odd (2n + 1) kernel sizes.")
        }
    }

    // Normalize data
    let mut max : f32 = vector[0];
    for i in &vector {
        if max < *i {
            max = *i;
        }
    }

    vector = vector.into_iter().map(| x | x / max).collect();

    // Moving average filter
    let mut temp_sum : f32;
    let mut temp_counter : u8;
    for i in 0..WINDOW_SIZE {
        temp_sum = 0.0;
        if (i >= (KERNEL_SIZE / 2) as usize && i < (WINDOW_SIZE - KERNEL_SIZE as usize / 2)) {
            temp_sum = vector[(i-(KERNEL_SIZE as usize)/2)..(i+(KERNEL_SIZE as usize)/2 + 1)].iter().sum();
            vector[i] = temp_sum / (KERNEL_SIZE as f32);
        } else {
            // edge cases
            temp_counter = 0;
            for ind in (i as i64 - (KERNEL_SIZE / 2) as i64)..(i as i64 + (KERNEL_SIZE / 2) as i64 + 1) {
                if (ind > 0) {
                    temp_sum += vector[i];
                    temp_counter += 1;
                }
            }

            /* Because the edges don't have enough values for a full kernel (either to the left or to the right)
                Instead of just leaving it to a smaller average, use a weighted average where the block itself has 
                KERNEL SIZE - ACTUAL AMPLITUDES IN KERNEL + 1 weightage.

                Example: if we are using the second to last element with KERNEL SIZE = 9:

                [1, 2, 3 ...... 90, 91, 92, 93, 94, 95, 96, 97, 98, _99_, 100]
                Around 99 we see only 100 to the right. KERNEL SIZE stays at 9, but the number of actual amplitudes 
                in the kernel are 4 to the right (98, 97, 96, 95), and 100 to the right. 

                ACTUAL AMPLITUDES IN KERNEL = 6 (inclusive with 99). 

                The weightage would be 95..98 + 99 + 100 + 99 * 4
             */
            vector[i] = (temp_sum + vector[i] * ((KERNEL_SIZE - temp_counter) as f32)) / KERNEL_SIZE as f32;
        }
    }

    vector
}

fn amplitude_rms(amplitudes:&Vec<f32>) -> f32 {

    // vector should have proper window size.
    match amplitudes.len().cmp(&(WINDOW_SIZE as usize)) {
        Ordering::Equal => { }
        Ordering::Greater => {} 
        Ordering::Less => {
            println!("Length is: {}", amplitudes.len());
            return -1.0;
        }
    }

    let mut rms : f32 = 0.0;
    for amp in amplitudes {
        rms += amp * amp / (WINDOW_SIZE as f32);
    }
    
    // return root of mean squared calculation of forloop
    rms.sqrt()
}

fn generate_gps_data() -> GpsData {
    let mut rng = rand::thread_rng();
    GpsData { time: (rng.gen()), 
              longitude: (rng.gen()), 
              lattitude: (rng.gen()), 
              altitude: (rng.gen()) }
}

fn store_data(audio_buffer : &mut [f32], start_time : &mut f32) {
    let mut f = File::create(format!("./data/audio_file_{}.txt", start_time.to_string())).expect("Failed!");
    // println!("./data/audio_file_{}.txt", start_time.to_string());
    let mut buffer = Vec::new();

    for i in audio_buffer {
        for byte in i.to_be_bytes() {
            buffer.push(byte);
        }
    }
    f.write(buffer.as_slice()).expect("Writing audio data caused error!");

    let mut gps_data : GpsData;
    let mut buffer: Vec<u8> = Vec::new();

    let now = Instant::now();
    loop { 
        gps_data = generate_gps_data();
        buffer = [buffer, gps_data.time.to_be_bytes().to_vec(), 
                            gps_data.altitude.to_be_bytes().to_vec(),
                            gps_data.longitude.to_be_bytes().to_vec(),
                            gps_data.lattitude.to_be_bytes().to_vec()].concat();
        thread::sleep(Duration::from_secs_f32(0.5));

        if (now.elapsed().as_secs_f32() > 5.0) {
            f.write(buffer.as_slice()).expect("Writing gps data caused error!");
            break;
        }
    }

    // Increment start_time (of the window) for proper file names.
    *start_time += WINDOW_SIZE as f32 / SAMPLE_RATE_HZ as f32;
}