use chrono::NaiveDate;

pub mod live;

pub async fn douyu(rid: Option<String>, date: Option<NaiveDate>) {
    evina_core::check_env("yt-dlp").await;
    live::douyu::down_his(rid, date).await
}
