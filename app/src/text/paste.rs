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
        let text = res.text().await.unwrap();
        println!("{}", text);
        // If you want to get the Location header:
        // (You may need to clone the response before reading the body if you want both)
        // let location = res.headers().get("Location").map(|l| l.to_str().unwrap());
        // println!("Location: {:?}", location);
    } else {
        println!("Error: {}", res.unwrap_err());
    }
}