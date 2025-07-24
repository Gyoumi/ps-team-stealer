use reqwest::header::CONTENT_TYPE;
use std::collections::HashMap;

pub async fn upload_team(team: String) {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let mut params = HashMap::new();
    params.insert("paste", team);

    let res = client
        .post("https://pokepasat.es/create")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await;

    if let Ok(res) = res {
        if res.status().is_redirection() {
            let location = res
                .headers()
                .get("Location")
                .and_then(|l| l.to_str().ok())
                .map(|s| format!("https://pokepasat.es{}", s));
            println!("Paste URL: {:?}", location);
        } else {
            println!("Unexpected response: {:?}", res.status());
        }
    } else {
        println!("Error: {}", res.unwrap_err());
    }
}