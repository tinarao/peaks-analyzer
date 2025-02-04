pub struct Job {
    id: i64,
    file_path: String,
    callback_api_url: String, // Where to POST when it's done
}

// impl Job {
//     pub fn new(file_path: String, callback_api_url: String) -> Job {
//         Job {
//             id: 0, // basically it means that Job is not persisted
//             file_path,
//             callback_api_url,
//         }
//     }

    // pub async fn persist(&mut self) -> Result<i64, String> {
    //     if self.id != 0 {
    //         return Err(String::from("Failed to persist already persisted job"));
    //     }

        // match self.execute_insert_query(db).await {
        //     Ok(r) => {
        //         let new_id = r.last_insert_rowid();
        //         self.id = new_id;
        //
        //         Ok(self.id)
        //     }
        //     Err(e) => {
        //         println!("Failed to run \"create\" sql query: {}", e);
        //         // Add some additional logging?
        //
        //         Err(String::from("Failed to persist a job."))
        //     }
        // }
    // }

    // async fn execute_insert_query(
    //     &self,
    //     mut db: Connection<Db>,
    // ) -> Result<SqliteQueryResult, sqlx::Error> {
    //     let result = sqlx::query(
    //         "
    //         INSERT INTO jobs (file_path, callback_api_url)
    //         VALUES (?, ?)
    //         ",
    //     )
    //     .bind(&self.file_path)
    //     .bind(&self.callback_api_url)
    //     .execute(&mut **db)
    //     .await;
    //
    //     return result;
    // }
// }
