#!/bin/bash

# Mock Service - Google Cloud Run Deployment Script
# Usage: ./deploy.sh <PROJECT_ID> [REGION] [SERVICE_NAME]

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
DEFAULT_REGION="us-central1"
DEFAULT_SERVICE_NAME="mock-service-nba"
DEFAULT_IMAGE_NAME="mock-service"

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to show usage
show_usage() {
    echo "Usage: $0 <PROJECT_ID> [REGION] [SERVICE_NAME]"
    echo ""
    echo "Arguments:"
    echo "  PROJECT_ID    : Google Cloud Project ID (required)"
    echo "  REGION        : Cloud Run region (default: $DEFAULT_REGION)"
    echo "  SERVICE_NAME  : Cloud Run service name (default: $DEFAULT_SERVICE_NAME)"
    echo ""
    echo "Examples:"
    echo "  $0 my-project-123"
    echo "  $0 my-project-123 us-east1"
    echo "  $0 my-project-123 europe-west1 nba-mock-prod"
    echo ""
    exit 1
}

# Check arguments
if [ $# -lt 1 ]; then
    print_error "Project ID is required!"
    show_usage
fi

# Parse arguments
PROJECT_ID="$1"
REGION="${2:-$DEFAULT_REGION}"
SERVICE_NAME="${3:-$DEFAULT_SERVICE_NAME}"
IMAGE_NAME="$DEFAULT_IMAGE_NAME"
FULL_IMAGE_NAME="gcr.io/$PROJECT_ID/$IMAGE_NAME"

print_info "Starting deployment with the following configuration:"
echo "  Project ID    : $PROJECT_ID"
echo "  Region        : $REGION"
echo "  Service Name  : $SERVICE_NAME"
echo "  Image Name    : $FULL_IMAGE_NAME"
echo ""

# Check required tools
print_info "Checking required tools..."

if ! command_exists gcloud; then
    print_error "Google Cloud CLI (gcloud) is not installed!"
    print_info "Install from: https://cloud.google.com/sdk/docs/install"
    exit 1
fi

if ! command_exists docker; then
    print_error "Docker is not installed!"
    print_info "Install from: https://docs.docker.com/get-docker/"
    exit 1
fi

print_success "All required tools are available"

# Check if user is authenticated
print_info "Checking Google Cloud authentication..."
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q .; then
    print_error "Not authenticated with Google Cloud!"
    print_info "Run: gcloud auth login"
    exit 1
fi

print_success "Google Cloud authentication verified"

# Set project
print_info "Setting Google Cloud project to $PROJECT_ID..."
if ! gcloud config set project "$PROJECT_ID"; then
    print_error "Failed to set project. Please check if project ID is correct."
    exit 1
fi

print_success "Project set successfully"

# Enable required APIs
print_info "Enabling required Google Cloud APIs..."
gcloud services enable \
    cloudbuild.googleapis.com \
    run.googleapis.com \
    containerregistry.googleapis.com

print_success "APIs enabled successfully"

# Configure Docker for GCR
print_info "Configuring Docker authentication for Google Container Registry..."
gcloud auth configure-docker --quiet

print_success "Docker authentication configured"

# Build Docker image
print_info "Building Docker image..."
print_info "This may take several minutes for the first build..."

if ! docker build -t "$FULL_IMAGE_NAME" .; then
    print_error "Docker build failed!"
    exit 1
fi

print_success "Docker image built successfully"

# Push image to GCR
print_info "Pushing image to Google Container Registry..."
if ! docker push "$FULL_IMAGE_NAME"; then
    print_error "Failed to push image to GCR!"
    exit 1
fi

print_success "Image pushed to GCR successfully"

# Deploy to Cloud Run
print_info "Deploying to Cloud Run..."
print_info "Service: $SERVICE_NAME"
print_info "Region: $REGION"

if ! gcloud run deploy "$SERVICE_NAME" \
    --image="$FULL_IMAGE_NAME" \
    --region="$REGION" \
    --platform=managed \
    --allow-unauthenticated \
    --port=8080 \
    --memory=512Mi \
    --cpu=1 \
    --min-instances=0 \
    --max-instances=10 \
    --timeout=300 \
    --concurrency=100 \
    --set-env-vars="RUST_LOG=info" \
    --quiet; then
    print_error "Cloud Run deployment failed!"
    exit 1
fi

print_success "Deployment completed successfully!"

# Get service URL
print_info "Getting service URL..."
SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" \
    --region="$REGION" \
    --format="value(status.url)")

if [ -z "$SERVICE_URL" ]; then
    print_warning "Could not retrieve service URL"
else
    print_success "Service deployed successfully!"
    echo ""
    echo "🚀 Your Mock Service NBA is now live at:"
    echo "   $SERVICE_URL"
    echo ""
    echo "📋 Test the deployment:"
    echo "   curl $SERVICE_URL/healthz"
    echo "   curl $SERVICE_URL/api/services"
    echo "   curl $SERVICE_URL/plan_de_ruta"
    echo ""
    echo "📊 View service details:"
    echo "   https://console.cloud.google.com/run/detail/$REGION/$SERVICE_NAME/metrics?project=$PROJECT_ID"
    echo ""
fi

# Test health endpoint
print_info "Testing health endpoint..."
if command_exists curl; then
    if curl -f -s "$SERVICE_URL/healthz" > /dev/null; then
        print_success "Health check passed! Service is responding correctly."
    else
        print_warning "Health check failed. Service might still be starting up."
        print_info "Wait a few seconds and try: curl $SERVICE_URL/healthz"
    fi
else
    print_warning "curl not available. Test manually: $SERVICE_URL/healthz"
fi

# Summary
echo ""
print_success "🎉 Deployment Summary:"
echo "  ✅ Project: $PROJECT_ID"
echo "  ✅ Region: $REGION" 
echo "  ✅ Service: $SERVICE_NAME"
echo "  ✅ URL: $SERVICE_URL"
echo ""
print_info "Next steps:"
echo "  1. Test all NBA endpoints"
echo "  2. Configure custom domain if needed"
echo "  3. Set up monitoring and alerting"
echo "  4. Configure CI/CD pipeline with cloudbuild.yaml"
echo ""

exit 0