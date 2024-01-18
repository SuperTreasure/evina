use chrono::NaiveDate;

pub mod live;

pub async fn douyu(rid: Option<String>, date: Option<NaiveDate>) {
    live::douyu::down_his(rid, date).await
}
