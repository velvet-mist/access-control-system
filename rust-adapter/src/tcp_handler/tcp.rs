use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct InspectionPayLoad{
    device_id: String,
    result: String,
    raw: String,
}

async fn forward_to_fastapi(payload: InspectionPayLoad) -> Result<(), Box<dyn Error>>{
    let client= Client::new();

    client
    .post("https://127.0.0.1:8000/inspection")
    .json(&payload)
    .send()
    .await?;

    Ok(())
}