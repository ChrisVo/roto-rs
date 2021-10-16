use clap::{App, Arg};
use ctrlc;
use hound;
use pv_recorder::{Recorder, RecorderBuilder};
use rodio::{Decoder, OutputStream, Source};
use std::fs::File;
use std::io::BufReader;
use std::io::{stdin, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

const SAMPLE_RATE: usize = 16000;

static LISTENING: AtomicBool = AtomicBool::new(false);

fn show_audio_devices() {
    println!("Printing audio devices...");
    let audio_devices = Recorder::get_audio_devices();
    match audio_devices {
        Ok(audio_devices) => {
            for (idx, device) in audio_devices.iter().enumerate() {
                println!("{}: {:?}", idx, device);
            }
        }
        Err(err) => panic!("Failed to get audio devices: {}", err),
    };
    println!("");
}

#[warn(dead_code)]
fn play_audio() {
    let (_stream, stream_handle) = OutputStream::try_default().expect("Can't get output stream");
    let file =
        BufReader::new(File::open("sounds/recording.mp3").expect("Can't read the recording"));
    let source = Decoder::new(file).expect("Couldn't decode given file");
    stream_handle
        .play_raw(source.convert_samples())
        .expect("Couldn't play audio");
    std::thread::sleep(std::time::Duration::from_secs(15));
}
fn main() {
    let matches = App::new("PvRecorder Demo")
        .arg(
            Arg::with_name("audio_device_index")
                .long("audio_device_index")
                .value_name("INDEX")
                .help("Index of input audio device.")
                .takes_value(true)
                .default_value("-1"),
        )
        .arg(
            Arg::with_name("output_path")
                .long("output_path")
                .value_name("PATH")
                .help("Path to write recorded audio wav file to.")
                .takes_value(true)
                .default_value("example.wav"),
        )
        .arg(Arg::with_name("show_audio_devices").long("show_audio_devices"))
        .get_matches();

    if matches.is_present("show_audio_devices") {
        return show_audio_devices();
    }

    let audio_device_index = matches
        .value_of("audio_device_index")
        .unwrap()
        .parse()
        .unwrap();

    let output_path = matches.value_of("output_path").unwrap();

    let recorder = RecorderBuilder::new()
        .device_index(audio_device_index)
        .init()
        .expect("Failed to initialize pvrecorder");
    ctrlc::set_handler(|| {
        LISTENING.store(false, Ordering::SeqCst);
    })
    .expect("Unable to setup signal handler");

    // Loop and wait for signal

    loop {
        println!("Starting the loop");
        for c in stdin().keys() {
            println!("Waiting for a key");
            match c.unwrap() {
                Key::Char(r) => {
                    println!("{} received", r);
                    break
                }
                _ => {}
            }
        }

        println!("Start recording...");
        recorder.start().expect("Failed to start audio recording");
        LISTENING.store(true, Ordering::SeqCst);

        let mut audio_data = Vec::new();
        while LISTENING.load(Ordering::SeqCst) {
            let mut frame_buffer = vec![0; recorder.frame_length()];
            recorder
                .read(&mut frame_buffer)
                .expect("Failed to read audio frame");
            audio_data.extend_from_slice(&frame_buffer);
        }

        println!("Stop recording...");
        recorder.stop().expect("Failed to stop audio recording");

        println!("Dumping audio to file...");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(output_path, spec).unwrap();
        for sample in audio_data {
            writer.write_sample(sample).unwrap();
        }
    }
}
