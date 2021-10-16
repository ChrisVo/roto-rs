use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

#[tokio::main]
async fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().expect("Can't get output stream");
    let file =
        BufReader::new(File::open("sounds/recording.mp3").expect("Can't read the recording"));
    let source = Decoder::new(file).expect("Couldn't decode given file");

    stream_handle.play_raw(source.convert_samples()).expect("Couldn't play audio");
    std::thread::sleep(std::time::Duration::from_secs(15));
}
