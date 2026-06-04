#[cfg(test)]
mod tests {
    use actix_web::{web, App, http::StatusCode};
    use std::sync::Arc;

    use crate::api::{ChatJimmyClient, ModelsResponse};
    use crate::server::handlers::{list_models, chat_completions, AppState};

    fn create_test_state() -> web::Data<AppState> {
        web::Data::new(AppState {
            client: Arc::new(ChatJimmyClient::new()),
        })
    }

    #[actix_web::test]
    async fn test_list_models() {
        let state = create_test_state();
        let app = App::new()
            .app_data(state.clone())
            .route("/v1/models", web::get().to(list_models))
            .route("/v1/chat/completions", web::post().to(chat_completions));
        let app = actix_web::test::init_service(app).await;
        let req = actix_web::test::TestRequest::get()
            .uri("/v1/models")
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = actix_web::test::read_body(resp).await;
        let models: ModelsResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(models.object, "list");
        assert_eq!(models.data.len(), 1);
        assert_eq!(models.data[0].id, "llama3.1-8B");
    }

    #[actix_web::test]
    async fn test_health_check() {
        let state = create_test_state();
        let app = App::new()
            .app_data(state.clone())
            .route("/health", web::get().to(crate::server::handlers::health_check))
            .route("/v1/models", web::get().to(list_models))
            .route("/v1/chat/completions", web::post().to(chat_completions));
        let app = actix_web::test::init_service(app).await;
        let req = actix_web::test::TestRequest::get()
            .uri("/health")
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = actix_web::test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "healthy");
        assert_eq!(json["service"], "blitz");
    }

    #[test]
    fn test_chat_completion_request_deserialize_with_stream() {
        let json = r#"{"model":"llama3.1-8B","messages":[{"role":"user","content":"hello"}],"stream":true}"#;
        let req: crate::api::ChatCompletionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, "llama3.1-8B");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.stream, Some(true));
    }

    #[test]
    fn test_chat_completion_request_deserialize_no_stream() {
        let json = r#"{"model":"llama3.1-8B","messages":[{"role":"user","content":"hello"}]}"#;
        let req: crate::api::ChatCompletionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.stream, None);
    }
}
