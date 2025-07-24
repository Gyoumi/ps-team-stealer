use reqwest::header::CONTENT_TYPE;
use std::collections::HashMap;

pub async fn upload_team(team: String) {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("paste", team);

    let res = client
        .post("https://pokepast.es/create")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await;

    if let Ok(res) = res {
        let location = res
            .headers()
            .get("Location")
            .and_then(|l| l.to_str().ok())
            .map(|s| s.to_string());

        let text = res.text().await.unwrap();
        println!("{}", text);
        println!("Location: {:?}", location.unwrap());
    } else {
        println!("Error: {}", res.unwrap_err());
    }
}