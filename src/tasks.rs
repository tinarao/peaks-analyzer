use crate::peaks::generate_peaks;
use serde::Serialize;
use std::collections::HashMap;
use tokio::{
    fs,
    sync::MutexGuard
};

#[derive(Serialize)]
pub struct Task {
    file_path: String,
    is_finished: bool,
    callback_api_url: String, // Where to POST when it's done
}

impl Task {
    pub fn new(file_path: String, callback_api_url: String) -> Task {
        Task {
            file_path,
            is_finished: false,
            callback_api_url,
        }
    }

    pub async fn complete(&self) -> Result<(), String> {
        println!("completing task");
        let peaks = generate_peaks(self.file_path.clone());

        let mut peaks_map = HashMap::new();
        peaks_map.insert("peaks", peaks);

        let client = reqwest::Client::new();
        let response = client
            .post(&self.callback_api_url)
            .json(&peaks_map)
            .send()
            .await;

        match response {
            Ok(r) => {
                println!("response status: {}", r.status());
            }
            Err(e) => {
                println!("{}", e);
                let err_text = format!("Не удалось сделать POST-запрос по callback_api_url: {}. Задача не будет удалена и будет исполняться заново при возможности.", &self.callback_api_url);

                return Err(err_text);
            }
        }

        match fs::remove_file(&self.file_path).await {
            Ok(()) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub async fn manage_tasks(mut tasks: MutexGuard<'_, Vec<Task>>) {
    for i in 0..tasks.len() {
        if !tasks[i].is_finished {
            println!("found uncompleted task!");
            match tasks[i].complete().await {
                Ok(_) => {
                    println!("Task completed. ");
                    tasks.remove(i);

                },
                Err(e) => println!("err: {}", e),
            };
        }
    }
}
