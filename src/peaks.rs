use proteus_audio::peaks::get_peaks;

pub fn generate_peaks(file_path: String) -> Vec<f32> {
    let peaks = get_peaks(&file_path, false);
    let mut result: Vec<f32> = Vec::new();

    let channel = &peaks[0];
    for i in 0..channel.len() {
        if i % 10 == 0 {
            result.push(channel[i].0);
        };
    }

    return result;
}
