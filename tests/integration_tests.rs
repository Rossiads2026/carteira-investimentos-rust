use reqwest::Client;

#[tokio::test]
async fn test_servidor_responde_login() {
    let client = Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    // Verificação de estrutura básica enviando requisição para a rota
    let response = client
        .get("http://127.0.0.1:3000/login")
        .send()
        .await;

    // Se o servidor estiver rodando, deve responder com status HTTP 200
    if let Ok(res) = response {
        assert_eq!(res.status(), 200);
    }
}