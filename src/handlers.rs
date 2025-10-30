use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;

use crate::utils::{
    read_mock_file, 
    get_services_list, 
    create_service_directory,
    save_json_file,
    delete_service_directory,
    MockError,
    ServiceRegistry,
    match_dynamic_route,
    process_dynamic_service
};

#[derive(Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub methods: Vec<String>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

/// Handle mock requests for services
pub async fn handle_mock_request(
    path: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let service_name = path.into_inner();
    let method = req.method().as_str();

    log::info!("Mock request: {} {}", method, service_name);

    match read_mock_file(&service_name, method) {
        Ok(content) => {
            log::info!("Serving mock response for {} {}", method, service_name);
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(content))
        }
        Err(MockError::FileNotFound(msg)) => {
            log::warn!("Mock file not found: {}", msg);
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(&msg)))
        }
        Err(MockError::ParseError(msg)) => {
            log::error!("JSON parse error: {}", msg);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&msg)))
        }
        Err(MockError::IoError(msg)) => {
            log::error!("IO error: {}", msg);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&msg)))
        }
    }
}

/// List all available services
pub async fn list_services() -> Result<HttpResponse> {
    log::info!("Listing all services");
    
    match get_services_list() {
        Ok(services) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(services)))
        }
        Err(e) => {
            log::error!("Error listing services: {}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error listing services: {}", e))))
        }
    }
}

/// Create a new service directory
pub async fn create_service(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let service_name = path.into_inner();
    
    log::info!("Creating service: {}", service_name);

    match create_service_directory(&service_name) {
        Ok(_) => {
            Ok(HttpResponse::Created().json(ApiResponse::success(format!("Service '{}' created successfully", service_name))))
        }
        Err(MockError::IoError(msg)) if msg.contains("already exists") => {
            Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(&format!("Service '{}' already exists", service_name))))
        }
        Err(e) => {
            log::error!("Error creating service {}: {}", service_name, e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error creating service: {}", e))))
        }
    }
}

/// Upload a mock file for a specific service and method
pub async fn upload_mock_file(
    path: web::Path<(String, String)>,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    let (service_name, method) = path.into_inner();
    let method = method.to_uppercase();
    
    log::info!("Uploading mock file for {} {}", method, service_name);

    // Validate HTTP method
    if !["GET", "POST", "PUT", "DELETE"].contains(&method.as_str()) {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid HTTP method. Must be GET, POST, PUT, or DELETE")));
    }

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        
        if let Some(filename) = content_disposition.and_then(|cd| cd.get_filename()) {
            if !filename.ends_with(".json") {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Only JSON files are allowed")));
            }
        }

        let mut file_content = Vec::new();
        while let Some(chunk) = field.try_next().await? {
            file_content.extend_from_slice(&chunk);
        }

        let json_str = String::from_utf8(file_content)
            .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UTF-8 content"))?;

        // Validate JSON
        let json_value: Value = serde_json::from_str(&json_str)
            .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid JSON: {}", e)))?;

        match save_json_file(&service_name, &method, &json_value) {
            Ok(_) => {
                log::info!("Mock file uploaded successfully for {} {}", method, service_name);
                return Ok(HttpResponse::Created().json(ApiResponse::success(format!("Mock file uploaded for {} {}", method, service_name))));
            }
            Err(e) => {
                log::error!("Error saving mock file: {}", e);
                return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error saving file: {}", e))));
            }
        }
    }

    Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("No file uploaded")))
}

/// Delete a service and all its mock files
pub async fn delete_service(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let service_name = path.into_inner();
    
    log::info!("Deleting service: {}", service_name);

    match delete_service_directory(&service_name) {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(format!("Service '{}' deleted successfully", service_name))))
        }
        Err(MockError::FileNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(&format!("Service '{}' not found", service_name))))
        }
        Err(e) => {
            log::error!("Error deleting service {}: {}", service_name, e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Error deleting service: {}", e))))
        }
    }
}

/// Health check endpoint
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "mock-service",
        "version": env!("CARGO_PKG_VERSION")
    })))
}

/// Handle dynamic requests with path parameters
pub async fn handle_dynamic_request(
    path: web::Path<String>,
    req: HttpRequest,
    registry: web::Data<Arc<ServiceRegistry>>,
) -> Result<HttpResponse> {
    let request_path = format!("/{}", path.into_inner());
    let method = req.method().as_str();
    
    log::debug!("Dynamic request: {} {}", method, request_path);
    
    // Try to match against dynamic routes
    if let Some((service_name, params)) = match_dynamic_route(&registry, &request_path, method) {
        log::info!("Matched dynamic route: {} -> service: {}, params: {:?}", request_path, service_name, params);
        
        if let Some(service_config) = registry.services.get(&service_name) {
            match process_dynamic_service(service_config, params, method) {
                Ok(content) => {
                    log::info!("Serving dynamic response for {} {}", method, request_path);
                    Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .json(content))
                }
                Err(MockError::FileNotFound(msg)) => {
                    log::warn!("Dynamic service error: {}", msg);
                    Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(&msg)))
                }
                Err(MockError::ParseError(msg)) => {
                    log::error!("Dynamic service parse error: {}", msg);
                    Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(&msg)))
                }
                Err(MockError::IoError(msg)) => {
                    log::error!("Dynamic service IO error: {}", msg);
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&msg)))
                }
            }
        } else {
            log::error!("Service configuration not found for: {}", service_name);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Service configuration error")))
        }
    } else {
        // Fallback to static service lookup for backward compatibility
        let path_parts: Vec<&str> = request_path.trim_start_matches('/').split('/').collect();
        
        if path_parts.len() == 1 && !path_parts[0].is_empty() {
            // Try legacy static service
            let service_name = path_parts[0];
            log::debug!("Trying legacy static service: {}", service_name);
            
            match read_mock_file(service_name, method) {
                Ok(content) => {
                    log::info!("Serving legacy static response for {} {}", method, service_name);
                    Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .json(content))
                }
                Err(_) => {
                    // Not found
                    Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(&format!(
                        "No mock service found for path: {} {}",
                        method, request_path
                    ))))
                }
            }
        } else {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(&format!(
                "No route configured for path: {} {}",
                method, request_path
            ))))
        }
    }
}