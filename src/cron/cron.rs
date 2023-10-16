use std::sync::Arc;

use crate::{
    cron::{clean_db_saves::clean_db_saves, save_dbs::save_dbs},
    AppState,
};

pub async fn start_cron(app_state: Arc<AppState>) -> () {
    println!("🚀 Starting cron");

    let daily = tokio::time::Duration::from_secs(60 * 60 * 24);

    let cron = tokio::spawn(async move {
        loop {
            if save_dbs(app_state.clone()).await.is_err() {
                println!("❌ Failed to run db cron");
            } else {
                println!("✅ Ran db cron");
            };
            if clean_db_saves(app_state.clone()).await.is_err() {
                println!("❌ Failed to clean dbs saves");
            } else {
                println!("✅ Cleaned dbs saves");
            };
            tokio::time::sleep(daily).await;
        }
    });

    cron.await.unwrap();
}
