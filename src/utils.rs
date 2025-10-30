use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::fmt;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use regex::Regex;
use chrono::{DateTime, Utc};
use handlebars::Handlebars;
use rquickjs::{Context, Runtime};
use once_cell::sync::Lazy;
use walkdir::WalkDir;

use crate::handlers::ServiceInfo;

const SERVICES_DIR: &str = "services";

// Global template engine
static HANDLEBARS: Lazy<Handlebars> = Lazy::new(|| {
    let mut hb = Handlebars::new();
    hb.set_strict_mode(true);
    hb
});

// Service configuration structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub pattern: String,
    pub method: String,
    pub params: HashMap<String, ParamConfig>,
    pub cache_ttl: Option<u64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamConfig {
    pub param_type: String,
    pub pattern: Option<String>,
    pub required: Option<bool>,
    pub default: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ServiceType {
    Static {
        content: Value,
    },
    Dynamic {
        template: Value,
        transformer: String,
        route_config: RouteConfig,
    },
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub service_type: ServiceType,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct ServiceRegistry {
    pub services: HashMap<String, ServiceConfig>,
    pub route_patterns: Vec<(Regex, String, String)>, // (regex, service_name, method)
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            route_patterns: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum MockError {
    FileNotFound(String),
    ParseError(String),
    IoError(String),
}

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            MockError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            MockError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for MockError {}

/// Discover and load all services from the services directory
pub fn discover_services() -> Result<ServiceRegistry, MockError> {
    let mut registry = ServiceRegistry::new();
    let services_path = Path::new(SERVICES_DIR);
    
    if !services_path.exists() {
        fs::create_dir_all(services_path)
            .map_err(|e| MockError::IoError(format!("Failed to create services directory: {}", e)))?;
        return Ok(registry);
    }

    for entry in WalkDir::new(services_path)
        .min_depth(1)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            let service_name = entry.file_name().to_string_lossy().to_string();
            let service_path = entry.path();
            
            log::debug!("Discovering service: {} at {:?}", service_name, service_path);
            
            match load_service_config(&service_name, service_path) {
                Ok(config) => {
                    // Register route patterns for dynamic services
                    if let ServiceType::Dynamic { route_config, .. } = &config.service_type {
                        if let Ok(regex) = convert_pattern_to_regex(&route_config.pattern) {
                            registry.route_patterns.push((
                                regex,
                                service_name.clone(),
                                route_config.method.clone(),
                            ));
                            log::info!("Registered dynamic route: {} {} -> {}", 
                                route_config.method, route_config.pattern, service_name);
                        }
                    }
                    
                    registry.services.insert(service_name.clone(), config);
                    log::info!("Loaded service: {}", service_name);
                }
                Err(e) => {
                    log::warn!("Failed to load service {}: {}", service_name, e);
                }
            }
        }
    }

    log::info!("Service discovery completed. Loaded {} services", registry.services.len());
    Ok(registry)
}

/// Load configuration for a specific service
fn load_service_config(service_name: &str, service_path: &Path) -> Result<ServiceConfig, MockError> {
    // Check if it's a dynamic service (has routes.json)
    let routes_file = service_path.join("routes.json");
    let template_file = service_path.join("template.json");
    let transformer_file = service_path.join("transformer.js");
    
    if routes_file.exists() && template_file.exists() && transformer_file.exists() {
        // Dynamic service
        log::debug!("Loading dynamic service: {}", service_name);
        
        let route_config: RouteConfig = serde_json::from_str(
            &fs::read_to_string(&routes_file)
                .map_err(|e| MockError::IoError(format!("Failed to read routes.json: {}", e)))?
        ).map_err(|e| MockError::ParseError(format!("Invalid routes.json: {}", e)))?;
        
        let template: Value = serde_json::from_str(
            &fs::read_to_string(&template_file)
                .map_err(|e| MockError::IoError(format!("Failed to read template.json: {}", e)))?
        ).map_err(|e| MockError::ParseError(format!("Invalid template.json: {}", e)))?;
        
        let transformer = fs::read_to_string(&transformer_file)
            .map_err(|e| MockError::IoError(format!("Failed to read transformer.js: {}", e)))?;
        
        Ok(ServiceConfig {
            name: service_name.to_string(),
            service_type: ServiceType::Dynamic {
                template,
                transformer,
                route_config,
            },
            path: service_path.to_path_buf(),
        })
    } else {
        // Static service - look for method-specific JSON files
        log::debug!("Loading static service: {}", service_name);
        
        // For now, we'll load the first JSON file we find as static content
        // This maintains backward compatibility
        let json_files: Vec<_> = fs::read_dir(service_path)
            .map_err(|e| MockError::IoError(format!("Failed to read service directory: {}", e)))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        
        if json_files.is_empty() {
            return Err(MockError::FileNotFound(format!("No JSON files found in service directory: {:?}", service_path)));
        }
        
        // Load the first JSON file as default content
        let first_file = &json_files[0];
        let content: Value = serde_json::from_str(
            &fs::read_to_string(first_file.path())
                .map_err(|e| MockError::IoError(format!("Failed to read JSON file: {}", e)))?
        ).map_err(|e| MockError::ParseError(format!("Invalid JSON: {}", e)))?;
        
        Ok(ServiceConfig {
            name: service_name.to_string(),
            service_type: ServiceType::Static { content },
            path: service_path.to_path_buf(),
        })
    }
}

/// Convert route pattern like "/plan-de-ruta/{ruta_id}/{fecha}" to regex
fn convert_pattern_to_regex(pattern: &str) -> Result<Regex, MockError> {
    let mut regex_pattern = pattern.to_string();
    
    // Replace {param} with named capture groups
    let param_regex = Regex::new(r"\{([^}]+)\}")
        .map_err(|e| MockError::ParseError(format!("Invalid parameter regex: {}", e)))?;
    
    regex_pattern = param_regex.replace_all(&regex_pattern, r"(?P<$1>[^/]+)").to_string();
    
    // Escape forward slashes and add anchors
    regex_pattern = format!("^{}$", regex_pattern.replace("/", r"\/"));
    
    Regex::new(&regex_pattern)
        .map_err(|e| MockError::ParseError(format!("Failed to compile route regex: {}", e)))
}

/// Process a dynamic service request with parameters
pub fn process_dynamic_service(
    service_config: &ServiceConfig,
    params: HashMap<String, String>,
    method: &str,
) -> Result<Value, MockError> {
    match &service_config.service_type {
        ServiceType::Dynamic { template, transformer, route_config } => {
            if route_config.method.to_uppercase() != method.to_uppercase() {
                return Err(MockError::FileNotFound(format!(
                    "Method {} not supported for service {}. Expected: {}",
                    method, service_config.name, route_config.method
                )));
            }
            
            // Validate parameters
            validate_parameters(&params, &route_config.params)?;
            
            // Apply template processing with Handlebars
            let template_with_params = apply_template_substitution(template, &params)?;
            
            // Execute JavaScript transformer
            execute_transformer(&template_with_params, transformer, &params)
        }
        ServiceType::Static { .. } => {
            Err(MockError::ParseError("Cannot process static service as dynamic".to_string()))
        }
    }
}

/// Validate request parameters against route configuration
fn validate_parameters(
    params: &HashMap<String, String>,
    param_configs: &HashMap<String, ParamConfig>,
) -> Result<(), MockError> {
    for (param_name, config) in param_configs {
        let required = config.required.unwrap_or(true);
        
        match params.get(param_name) {
            Some(value) => {
                // Validate parameter format
                if let Some(pattern) = &config.pattern {
                    let regex = Regex::new(pattern)
                        .map_err(|e| MockError::ParseError(format!("Invalid parameter regex: {}", e)))?;
                    
                    if !regex.is_match(value) {
                        return Err(MockError::ParseError(format!(
                            "Parameter '{}' value '{}' doesn't match pattern '{}'",
                            param_name, value, pattern
                        )));
                    }
                }
                
                // Validate parameter type
                match config.param_type.as_str() {
                    "date" => {
                        if chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err() {
                            return Err(MockError::ParseError(format!(
                                "Parameter '{}' must be a valid date in YYYY-MM-DD format",
                                param_name
                            )));
                        }
                    }
                    "number" => {
                        if value.parse::<f64>().is_err() {
                            return Err(MockError::ParseError(format!(
                                "Parameter '{}' must be a valid number",
                                param_name
                            )));
                        }
                    }
                    "string" => {
                        // String validation already done by pattern if provided
                    }
                    _ => {
                        log::warn!("Unknown parameter type: {}", config.param_type);
                    }
                }
            }
            None => {
                if required {
                    return Err(MockError::ParseError(format!(
                        "Required parameter '{}' is missing",
                        param_name
                    )));
                }
            }
        }
    }
    
    Ok(())
}

/// Apply Handlebars template substitution
fn apply_template_substitution(
    template: &Value,
    params: &HashMap<String, String>,
) -> Result<Value, MockError> {
    let template_str = serde_json::to_string(template)
        .map_err(|e| MockError::ParseError(format!("Failed to serialize template: {}", e)))?;
    
    let rendered = HANDLEBARS.render_template(&template_str, params)
        .map_err(|e| MockError::ParseError(format!("Template rendering failed: {}", e)))?;
    
    serde_json::from_str(&rendered)
        .map_err(|e| MockError::ParseError(format!("Invalid JSON after template substitution: {}", e)))
}

/// Execute JavaScript transformer
fn execute_transformer(
    template: &Value,
    transformer_code: &str,
    params: &HashMap<String, String>,
) -> Result<Value, MockError> {
    let rt = Runtime::new()
        .map_err(|e| MockError::ParseError(format!("Failed to create JS runtime: {}", e)))?;
    
    let ctx = Context::full(&rt)
        .map_err(|e| MockError::ParseError(format!("Failed to create JS context: {}", e)))?;
    
    ctx.with(|ctx| -> Result<Value, MockError> {
        // Inject template and params into JS context
        let template_str = serde_json::to_string(template)
            .map_err(|e| MockError::ParseError(format!("Failed to serialize template: {}", e)))?;
        
        let params_str = serde_json::to_string(params)
            .map_err(|e| MockError::ParseError(format!("Failed to serialize params: {}", e)))?;
        
        // Create the complete JS code
        let js_code = format!(
            r#"
            const template = {};
            const params = {};
            const context = {{
                timestamp: new Date().toISOString(),
                requestId: Math.random().toString(36).substr(2, 9)
            }};
            
            {}
            
            // Ensure transform function exists
            if (typeof transform !== 'function') {{
                throw new Error('transform function not defined in transformer');
            }}
            
            // Execute transformation
            const result = transform(template, params, context);
            JSON.stringify(result);
            "#,
            template_str, params_str, transformer_code
        );
        
        // Execute JavaScript code
        let result: String = ctx.eval(js_code.as_bytes())
            .map_err(|e| MockError::ParseError(format!("JavaScript execution failed: {}", e)))?;
        
        // Parse result back to JSON
        serde_json::from_str(&result)
            .map_err(|e| MockError::ParseError(format!("Invalid JSON returned from transformer: {}", e)))
    })
}

/// Match request path against dynamic route patterns
pub fn match_dynamic_route(
    registry: &ServiceRegistry,
    path: &str,
    method: &str,
) -> Option<(String, HashMap<String, String>)> {
    for (regex, service_name, route_method) in &registry.route_patterns {
        if route_method.to_uppercase() == method.to_uppercase() {
            if let Some(captures) = regex.captures(path) {
                let mut params = HashMap::new();
                
                for name in regex.capture_names().flatten() {
                    if let Some(value) = captures.name(name) {
                        params.insert(name.to_string(), value.as_str().to_string());
                    }
                }
                
                return Some((service_name.clone(), params));
            }
        }
    }
    
    None
}

/// Read a mock file for a given service and HTTP method
pub fn read_mock_file(service_name: &str, method: &str) -> Result<Value, MockError> {
    let filename = format!("{}-{}.json", service_name, method.to_uppercase());
    let file_path = PathBuf::from(SERVICES_DIR)
        .join(service_name)
        .join(&filename);

    log::debug!("Looking for mock file: {:?}", file_path);

    if !file_path.exists() {
        return Err(MockError::FileNotFound(format!(
            "Mock file not found: {} for service '{}' and method '{}'",
            filename, service_name, method
        )));
    }

    let content = fs::read_to_string(&file_path)
        .map_err(|e| MockError::IoError(format!("Failed to read file {:?}: {}", file_path, e)))?;

    serde_json::from_str(&content)
        .map_err(|e| MockError::ParseError(format!("Invalid JSON in file {:?}: {}", file_path, e)))
}

/// Get a list of all available services
pub fn get_services_list() -> Result<Vec<ServiceInfo>, MockError> {
    let services_path = Path::new(SERVICES_DIR);
    
    if !services_path.exists() {
        fs::create_dir_all(services_path)
            .map_err(|e| MockError::IoError(format!("Failed to create services directory: {}", e)))?;
        return Ok(vec![]);
    }

    let mut services = Vec::new();

    let entries = fs::read_dir(services_path)
        .map_err(|e| MockError::IoError(format!("Failed to read services directory: {}", e)))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| MockError::IoError(format!("Failed to read directory entry: {}", e)))?;
        
        let path = entry.path();
        if path.is_dir() {
            let service_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
                .to_string();

            let methods = get_service_methods(&service_name)?;
            
            services.push(ServiceInfo {
                name: service_name,
                methods,
            });
        }
    }

    services.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(services)
}

/// Get available HTTP methods for a service
fn get_service_methods(service_name: &str) -> Result<Vec<String>, MockError> {
    let service_path = PathBuf::from(SERVICES_DIR).join(service_name);
    let mut methods = Vec::new();

    let entries = fs::read_dir(&service_path)
        .map_err(|e| MockError::IoError(format!("Failed to read service directory {:?}: {}", service_path, e)))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| MockError::IoError(format!("Failed to read directory entry: {}", e)))?;
        
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
                if filename.ends_with(".json") {
                    // Extract method from filename pattern: service_name-METHOD.json
                    let prefix = format!("{}-", service_name);
                    if filename.starts_with(&prefix) && filename.len() > prefix.len() + 5 {
                        let method = filename[prefix.len()..filename.len()-5].to_string();
                        if ["GET", "POST", "PUT", "DELETE"].contains(&method.as_str()) {
                            methods.push(method);
                        }
                    }
                }
            }
        }
    }

    methods.sort();
    Ok(methods)
}

/// Create a new service directory
pub fn create_service_directory(service_name: &str) -> Result<(), MockError> {
    let service_path = PathBuf::from(SERVICES_DIR).join(service_name);
    
    if service_path.exists() {
        return Err(MockError::IoError(format!("Service directory already exists: {:?}", service_path)));
    }

    fs::create_dir_all(&service_path)
        .map_err(|e| MockError::IoError(format!("Failed to create service directory {:?}: {}", service_path, e)))?;

    log::info!("Created service directory: {:?}", service_path);
    Ok(())
}

/// Save a JSON file for a service and method
pub fn save_json_file(service_name: &str, method: &str, content: &Value) -> Result<(), MockError> {
    let service_path = PathBuf::from(SERVICES_DIR).join(service_name);
    
    // Create service directory if it doesn't exist
    if !service_path.exists() {
        fs::create_dir_all(&service_path)
            .map_err(|e| MockError::IoError(format!("Failed to create service directory {:?}: {}", service_path, e)))?;
    }

    let filename = format!("{}-{}.json", service_name, method.to_uppercase());
    let file_path = service_path.join(&filename);

    let json_string = serde_json::to_string_pretty(content)
        .map_err(|e| MockError::ParseError(format!("Failed to serialize JSON: {}", e)))?;

    fs::write(&file_path, json_string)
        .map_err(|e| MockError::IoError(format!("Failed to write file {:?}: {}", file_path, e)))?;

    log::info!("Saved mock file: {:?}", file_path);
    Ok(())
}

/// Delete a service directory and all its files
pub fn delete_service_directory(service_name: &str) -> Result<(), MockError> {
    let service_path = PathBuf::from(SERVICES_DIR).join(service_name);
    
    if !service_path.exists() {
        return Err(MockError::FileNotFound(format!("Service directory not found: {:?}", service_path)));
    }

    fs::remove_dir_all(&service_path)
        .map_err(|e| MockError::IoError(format!("Failed to delete service directory {:?}: {}", service_path, e)))?;

    log::info!("Deleted service directory: {:?}", service_path);
    Ok(())
}

/// Validate service name (alphanumeric and underscores only)
pub fn validate_service_name(name: &str) -> bool {
    !name.is_empty() 
        && name.len() <= 50 
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
        && !name.starts_with('_')
        && !name.ends_with('_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_service_name() {
        assert!(validate_service_name("user_service"));
        assert!(validate_service_name("api_v1"));
        assert!(validate_service_name("test123"));
        
        assert!(!validate_service_name(""));
        assert!(!validate_service_name("_invalid"));
        assert!(!validate_service_name("invalid_"));
        assert!(!validate_service_name("invalid-name"));
        assert!(!validate_service_name("invalid name"));
    }
}