use proteus_audio::peaks::get_peaks;
use std::{fs::File, io::Write};

pub fn generate_peaks() {
    let peaks = get_peaks("test_audio.mp3", false);

    let file = File::create("result.txt").unwrap();

    let channel = &peaks[0];
    for i in 0..channel.len() {
        if i % 10 == 0 {
            let peak = channel[i].0.to_string();
            let _ = write!(&file, "{peak}\n");
        };
    }
}
