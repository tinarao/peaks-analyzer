use crate::peaks::generate_peaks;
use serde::Serialize;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{FromRow, Pool, Sqlite};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

#[derive(Serialize, FromRow)]
pub struct Task {
    id: i64,
    file_path: String,
    callback_api_url: String, // Where to POST when it's done
}

impl Task {
    pub fn new(file_path: String, callback_api_url: String) -> Task {
        Task {
            id: 0, // basically it means that Task is not persisted
            file_path,
            callback_api_url,
        }
    }

    pub async fn persist(&mut self, pool: Arc<Pool<Sqlite>>) -> Result<i64, String> {
        if self.id != 0 {
            return Err(String::from("Failed to persist already persisted job"));
        }

        match self.execute_insert_query(pool).await {
            Ok(v) => {
                let new_id = v.last_insert_rowid();
                self.id = new_id;

                Ok(new_id)
            }
            Err(e) => {
                println!("Failed to persist job: {}", e);

                Err(e.to_string())
            }
        }
    }

    async fn execute_insert_query(
        &self,
        mut pool: Arc<Pool<Sqlite>>,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        let result = sqlx::query(
            "
            INSERT INTO tasks (file_path, callback_api_url)
            VALUES (?, ?)
            ",
        )
        .bind(&self.file_path)
        .bind(&self.callback_api_url)
        .execute(&*pool)
        .await;

        return result;
    }

    pub async fn delete_row(&self, pool: Arc<Pool<Sqlite>>) -> Result<(), String> {
        match sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(self.id)
            .execute(&*pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Failed to remove task: {}", e);
                Err(e.to_string())
            }
        }
    }

    pub async fn complete(&self, pool: Arc<Pool<Sqlite>>) -> Result<(), String> {
        let peaks = generate_peaks(self.file_path.clone());

        // let resp = reqwest::RequestBuilder().to(self.callback_api_url).send();

        fs::remove_file(&self.file_path).await.unwrap();
        self.delete_row(pool).await
    }

    pub fn debug_display(&self) {
        println!(
            "Task №{}\nFilepath: {}\nCallback Url: {}\n\n",
            self.id, self.file_path, self.callback_api_url
        );
    }
}

pub async fn manage_tasks(pool: Arc<Pool<Sqlite>>) {
    let tasks = sqlx::query_as::<_, Task>("SELECT * FROM tasks")
        .fetch_all(&*pool)
        .await
        .expect("Failed to retrieve tasks");

    if tasks.len() > 0 {
        match tasks[0].complete(pool).await {
            Ok(_) => {
                println!("Completed task #{}", tasks[0].id)
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}

pub async fn remove_finished_tasks(pool: Arc<Pool<Sqlite>>) {
    let tasks = sqlx::query_as::<_, Task>("SELECT * FROM tasks")
        .fetch_all(&*pool)
        .await
        .expect("Failed to retrieve tasks");

    for task in tasks {
        let is_file_exists = Path::new("/etc/hosts").exists();
        if !is_file_exists {
            match task.delete_row(pool.clone()).await {
                Ok(_) => {
                    println!("Task №{} was deleted", task.id);
                }
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }
}
