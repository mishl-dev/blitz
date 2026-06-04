use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, middleware::Logger};
use futures::StreamExt;
use std::sync::Arc;
use tracing::{info, error};

use crate::api::{ChatCompletionRequest, ChatJimmyClient, ModelsResponse};

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<ChatJimmyClient>,
}

pub async fn run_server() -> std::io::Result<()> {
    info!("Server starting at http://0.0.0.0:3000");
    
    let app_state = AppState {
        client: Arc::new(ChatJimmyClient::new()),
    };

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::JsonConfig::default().limit(4_194_304)) // 4MB
            .app_data(web::Data::new(app_state.clone()))
            .route("/health", web::get().to(health_check))
            .route("/v1/models", web::get().to(list_models))
            .route("/v1/chat/completions", web::post().to(chat_completions))
    })
    .bind("0.0.0.0:3000")?
    .workers(num_cpus::get())
    .run()
    .await
}

pub(crate) async fn health_check() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .json(serde_json::json!({
            "status": "healthy",
            "service": "blitz"
        }))
}

pub(crate) async fn list_models() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .json(ModelsResponse::new())
}

pub(crate) async fn chat_completions(
    state: web::Data<AppState>,
    body: web::Json<ChatCompletionRequest>,
) -> HttpResponse {
    let request = body.into_inner();
    let client = state.client.clone();
    
    if request.stream.unwrap_or(false) {
        match client.complete_stream(&request).await {
            Ok(stream) => {
                let sse_stream = stream.map(|chunk| {
                    match chunk {
                        Ok(c) => {
                            let data = serde_json::to_string(&c).unwrap_or_default();
                            Ok::<_, std::io::Error>(web::Bytes::from(format!("data: {}\n\n", data)))
                        }
                        Err(e) => {
                            let data = serde_json::json!({"error": {"message": e}});
                            Ok::<_, std::io::Error>(web::Bytes::from(format!("data: {}\n\n", data)))
                        }
                    }
                });

                HttpResponse::Ok()
                    .content_type("text/event-stream")
                    .insert_header(("Cache-Control", "no-cache"))
                    .insert_header(("Connection", "keep-alive"))
                    .insert_header(("X-Accel-Buffering", "no"))
                    .streaming(sse_stream)
            }
            Err(e) => {
                error!("Error calling chatjimmy.ai: {}", e);
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "error": {
                            "message": e.to_string(),
                            "type": "internal_error"
                        }
                    }))
            }
        }
    } else {
        match client.complete(&request).await {
            Ok(response) => HttpResponse::Ok()
                .content_type("application/json")
                .json(response),
            Err(e) => {
                error!("Error calling chatjimmy.ai: {}", e);
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "error": {
                            "message": e.to_string(),
                            "type": "internal_error"
                        }
                    }))
            }
        }
    }
}
