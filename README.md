# Mock Service - Mock Endpoint Server in Rust

A web application developed in Rust with Actix Web that provides mock endpoints for testing and development. It allows you to serve predefined JSON responses and dynamically manage mock services.

## 🚀 Features

- **Mock Endpoints**: Serve JSON responses for `GET`, `POST`, `PUT`, `DELETE`
- **Dynamic Management**: API to create, list, and delete mock services
- **File Uploads**: Endpoints to upload JSON files as mock responses
- **CORS Enabled**: Configured to allow requests from any origin
- **Logging**: Detailed log system for debugging
- **Error Handling**: Structured responses for 404 and 500 errors
- **Validation**: Validation of JSON files and service names

## 📁 Project Structure

```
mock-service/
├── src/
│ ├── main.rs # Server entry point and configuration
│ ├── handlers.rs # Endpoint handlers
│ └── utils.rs # Helper functions
├── services/ # Mock service directory
│ ├── user_service/ # Example: User service
│ │ ├── user_service-GET.json
│ │ ├── user_service-POST.json
│ │ ├── user_service-PUT.json
│ │ └── user_service-DELETE.json
│ ├── product_service/ # Example: Product service
│ │ ├── product_service-GET.json
│ │ └── product_service-POST.json
│ ├── plan_de_ruta/    # NBA: Route plan with recommendations
│ │ └── plan_de_ruta-GET.json
│ ├── feedback/        # NBA: Recommendation feedback system
│ │ ├── feedback-GET.json
│ │ └── feedback-POST.json
│ ├── analytics/       # NBA: Analytics and metrics
│ │ └── analytics-GET.json
│ ├── plan_ruta_avanzado/ # NBA: Advanced route planning
│ │ └── plan_ruta_avanzado-GET.json
│ └── healthz/         # Health check endpoint
│     └── healthz-GET.json
├── Cargo.toml
└── README.md
```

## 🛠️ Installation and Configuration

### Prerequisites

- Rust 1.70+ installed
- Cargo (included with Rust)

### Installation

1. Clone the repository (or create the project):
```bash
git clone <repository-url>
cd mock-service
```

2. Build the project:
```bash
cargo build --release
```

3. Run the server:
```bash
cargo run
```

Configuration Options:
```bash
# Run on custom port
cargo run -- --port 3000

# Run on specific host
cargo run -- --host 0.0.0.0 --port 8080

# View help
cargo run -- --help
```

## 🎯 NBA Services (Next Best Action)

The system includes specialized mock services for the NBA (Next Best Action) recommendation engine based on the PQSM10 specification:

### Available NBA Endpoints

#### 1. **Route Planning** (`/plan_de_ruta`)
```bash
# Get enriched route plan with clients and recommendations
curl -X GET http://localhost:8080/plan_de_ruta
```

**Response structure**: Complete route plan with prioritized recommendations including:
- `ALERTA_QUIEBRE_STOCK` - Stock shortage alerts
- `SUGERENCIA_PORTAFOLIO` - Portfolio suggestions
- `OFERTA_DINAMICA` - Dynamic offers
- `ARGUMENTO_VENTA` - Sales arguments
- `INICIATIVA_CONVERSION_NR` - Non-regular client conversion
- `PEDIDO_OPTIMO` - Optimal order suggestions
- `SALUDO_CONSULTIVO` - Consultative greetings
- `RECUPERACION_VOLUMEN` - Volume recovery initiatives

#### 2. **Feedback System** (`/feedback`)
```bash
# Submit recommendation feedback
curl -X POST http://localhost:8080/feedback \
  -H "Content-Type: application/json" \
  -d '{
    "feedback_id": "fb-001",
    "plan_id": "P012-2025-10-22",
    "cliente_id": "C00123",
    "asesor_id": "A-77", 
    "recomendacion_id": "rec-1",
    "resultado_seleccionado": "Vendió",
    "comentario": "Cliente muy receptivo"
  }'

# Get feedback history
curl -X GET http://localhost:8080/feedback
```

#### 3. **Analytics Dashboard** (`/analytics`) 
```bash
# Get recommendation effectiveness metrics
curl -X GET http://localhost:8080/analytics
```

**Provides**: Performance metrics, success rates by recommendation type, advisor effectiveness, and trend analysis.

#### 4. **Advanced Route Planning** (`/plan_ruta_avanzado`)
```bash
# Get advanced route with recovery strategies
curl -X GET http://localhost:8080/plan_ruta_avanzado
```

#### 5. **Health Check** (`/healthz`)
```bash
# System health and status
curl -X GET http://localhost:8080/healthz
```

### NBA Testing Examples

```bash
# Test complete NBA workflow
echo "1. Get route plan"
curl -s http://localhost:8080/plan_de_ruta | jq '.resumen_ruta'

echo "2. Submit feedback"
curl -s -X POST http://localhost:8080/feedback \
  -H "Content-Type: application/json" \
  -d '{
    "feedback_id": "test-001",
    "plan_id": "P012-2025-10-22", 
    "cliente_id": "C00123",
    "asesor_id": "A-77",
    "recomendacion_id": "rec-1",
    "resultado_seleccionado": "Vendió"
  }' | jq '.status'

echo "3. Check analytics"
curl -s http://localhost:8080/analytics | jq '.data.metricas_generales'

echo "4. Health check"
curl -s http://localhost:8080/healthz | jq '.status'
```

### NBA Recommendation Types

The system supports 8 types of recommendations based on the PQSM10 specification:

| Type | Description | Priority Levels | Typical Use Case |
|------|-------------|----------------|------------------|
| `ALERTA_QUIEBRE_STOCK` | Stock shortage alerts | Alta, Crítica | Prevent stockouts on high-rotation products |
| `SUGERENCIA_PORTAFOLIO` | Portfolio suggestions | Media, Alta | Cross-selling and complementary products |
| `OFERTA_DINAMICA` | Dynamic offers | Alta | Promotional campaigns and volume incentives |
| `ARGUMENTO_VENTA` | Sales arguments | Baja, Media | Support for premium product positioning |
| `INICIATIVA_CONVERSION_NR` | Non-regular conversion | Media | Win back inactive clients |
| `PEDIDO_OPTIMO` | Optimal order size | Alta | Maximize order efficiency and client satisfaction |
| `SALUDO_CONSULTIVO` | Consultative greeting | Alta | Relationship building and needs assessment |
| `RECUPERACION_VOLUMEN` | Volume recovery | Alta, Crítica | Restore historical purchase volumes |

Each recommendation includes:
- **Payload**: Type-specific data (SKUs, amounts, incentives, etc.)
- **Feedback Config**: Available response options for advisors  
- **Priority Level**: Execution urgency (baja, media, alta, crítica)

## 🔧 Using the System

### Mock Endpoints

Mock endpoints follow the pattern: `HTTP_METHOD /<service_name>`

**Examples**:
```bash
GET http://localhost:8080/user_service # Returns user_service-GET.json
POST http://localhost:8080/user_service # Returns user_service-POST.json
PUT http://localhost:8080/user_service # Returns user_service-PUT.json
DELETE http://localhost:8080/user_service # Returns user_service-DELETE.json
```

### Management API

#### 1. **List all services**
```bash
curl -X GET http://localhost:8080/api/services
```

**Answer**:
```json
{ 
"success": true, 
"data": [ 
{ 
"name": "user_service", 
"methods": ["GET", "POST", "PUT", "DELETE"] 
}, 
{ 
"name": "product_service", 
"methods": ["GET", "POST"] 
} 
]
}
```

#### 2. **Create a new service**
```bash
curl -X POST http://localhost:8080/api/services/payment_service
```

#### 3. **Upload JSON file for a specific method**
```bash
# Create local JSON file
echo '{"status": "success", "data": {"payment_id": "12345"}}' > payment-get.json

# Upload file
curl -X PUT \
-F "file=@payment-get.json" \
http://localhost:8080/api/services/payment_service/GET
```

#### 4. **Delete a service**
```bash
curl -X DELETE http://localhost:8080/api/services/payment_service
```

#### 5. **Health Check**
```bash
curl -X GET http://localhost:8080/api/health
```

## 📝 Manually Creating JSON Files

### Directory Structure

For each service, create a directory in `services/` with JSON files named `<service>-<METHOD>.json`:

```bash
# Create directory for new service
mkdir -p services/auth_service

# Create response files
touch services/auth_service/auth_service-GET.json
touch services/auth_service/auth_service-POST.json
```

### Examples of JSON Files

**`services/auth_service/auth_service-POST.json`** (Login):
```json
{ 
"status": "success", 
"message": "Login successful", 
"data": { 
"user_id": 12345, 
"username": "admin", 
"token": "jwt-token-example", 
"expires_in": 3600, 
"permissions": ["read", "write", "admin"] 
}
}
```

**`services/auth_service/auth_service-GET.json`** (Check token):
```json
{
"status": "success",
"data": {
"valid": true,
"user_id": 12345,
"username": "admin",
"expires_at": "2024-01-19T18:30:00Z"
}
}
```

## 🧪 Testing Examples with curl

### Mock Services

```bash
# Get list of users
curl -X GET http://localhost:8080/user_service

# Create a new user
curl -X POST http://localhost:8080/user_service \
-H "Content-Type: application/json" \
-d '{"name": "New User", "email": "new@example.com"}'

# Update user
curl -X PUT http://localhost:8080/user_service
-H "Content-Type: application/json"
-d '{"id": 1, "name": "User Updated"}'

# Delete user
curl -X DELETE http://localhost:8080/user_service

# Test for non-existent service (404 response)
curl -X GET http://localhost:8080/nonexistent_service
```

### Service Management

```bash
# View all available services
curl -X GET http://localhost:8080/api/services | jq

# Create notification service
curl -X POST http://localhost:8080/api/services/notification_service

# Create JSON file for GET
echo '{
"status": "success",
"data": {
"notifications": [
{"id": 1, "message": "Welcome to the system", "read": false},
{"id": 2, "message": "Your profile has been updated", "read": true}
],
"unread_count": 1
}
}' > notification-get.json

# Upload the file
curl -X PUT \
-F "file=@notification-get.json" \
http://localhost:8080/api/services/notification_service/GET

# Test the new endpoint
curl -X GET http://localhost:8080/notification_service | jq

# Clear temporary file
rm notification-get.json
```

## 🔍 Error Responses

### 404 - Mock File Not Found
```json
{
"success": false,
"error": "Mock file not found: user_service-PATCH.json for service 'user_service' and method 'PATCH'"
}
```

### 400 - Validation Error
```json
{
"success": false,
"error": "Invalid HTTP method. Must be GET, POST, PUT, or DELETE"
}
```

### 500 - Server Error
```json
{
"success": false,
"error": "Error saving file: Permission denied"
}
```

## 🚀 Development and Extension

### Add New Endpoints

To add new endpoints in `src/main.rs`:

```rust
App::new()
.wrap(cors)
.wrap(Logger::default())
// Your new endpoints here
.route("/api/custom", web::get().to(handlers::custom_handler))
// Existing endpoints...
```

### Customizing Responses

Modify `src/handlers.rs` to add custom logic:

```rust
pub async fn custom_handler() -> Result<HttpResponse> {
// Your custom logic
Ok(HttpResponse::Ok().json(serde_json::json!({
"message": "Custom Endpoint"
})))
}
```

### Environment Variables

You can set the Logging using environment variables:

```bash
# Set log level
export RUST_LOG=debug
cargo run

# Application logs only
export RUST_LOG=mock_service=info
cargo run
```

## 📊 Logs and Monitoring

The system generates detailed logs:

```
[2024-01-18T20:30:15Z INFO mock_service] Starting Mock Service on 127.0.0.1:8080
[2024-01-18T20:30:20Z INFO mock_service::handlers] Mock request: GET user_service
[2024-01-18T20:30:20Z INFO mock_service::handlers] Serving mock response for GET user_service
[2024-01-18T20:30:25Z WARN mock_service::handlers] Mock file not found: payment_service-GET.json for service 'payment_service' and method 'GET'
```

## 🔒 Security Considerations

- **Development Only**: This service is designed for development and testing environments
- **No Authentication**: Does not include authentication mechanisms
- **Input Validation**: Validates JSON format but not specific content
- **File Limits**: There are no size limits for uploaded files

## 🤝 Contributions

1. Fork the project
2. Create a branch for your feature (`git checkout -b feature/new-feature`)
3. Commit your changes (`git commit -am 'Add new feature'`)
4. Push to the branch (`git push origin feature/new-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License. See the `LICENSE` file for details.

---

**Developed with ❤️ in Rust** 🦀