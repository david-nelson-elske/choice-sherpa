#!/bin/bash
# =============================================================================
# Zitadel OIDC Setup Script for Choice Sherpa
# =============================================================================
# This script configures Zitadel with the necessary OIDC applications for
# Choice Sherpa development environment.
#
# Prerequisites:
#   - Docker containers running (docker-compose up -d)
#   - curl and jq installed
#
# Usage:
#   ./scripts/setup-zitadel.sh
# =============================================================================

set -e

ZITADEL_URL="http://localhost:8085"
ADMIN_USER="admin"
ADMIN_PASS="Admin123!"
PROJECT_NAME="choice-sherpa"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# =============================================================================
# Wait for Zitadel to be ready
# =============================================================================
wait_for_zitadel() {
    log_info "Waiting for Zitadel to be ready..."
    local max_attempts=60
    local attempt=1

    while [ $attempt -le $max_attempts ]; do
        if curl -sf "${ZITADEL_URL}/debug/ready" > /dev/null 2>&1; then
            log_info "Zitadel is ready!"
            return 0
        fi
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done

    log_error "Zitadel failed to become ready after ${max_attempts} attempts"
    exit 1
}

# =============================================================================
# Get access token via password grant (for initial setup)
# =============================================================================
get_admin_token() {
    log_info "Authenticating as admin..."

    # First, we need to get the PAT or use the human login
    # For initial setup, we'll use the session API

    local response=$(curl -sf "${ZITADEL_URL}/oauth/v2/token" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "grant_type=password" \
        -d "username=${ADMIN_USER}" \
        -d "password=${ADMIN_PASS}" \
        -d "scope=openid profile urn:zitadel:iam:org:project:id:zitadel:aud" \
        2>/dev/null || echo "")

    if [ -z "$response" ]; then
        log_error "Failed to authenticate. Make sure Zitadel is running and credentials are correct."
        log_info "You may need to configure the application manually via the Zitadel Console."
        log_info "Access the console at: ${ZITADEL_URL}/ui/console"
        exit 1
    fi

    ACCESS_TOKEN=$(echo "$response" | jq -r '.access_token // empty')

    if [ -z "$ACCESS_TOKEN" ]; then
        log_warn "Password grant not enabled. Using console setup instructions instead."
        print_manual_setup
        exit 0
    fi

    log_info "Successfully authenticated!"
}

# =============================================================================
# Create project
# =============================================================================
create_project() {
    log_info "Creating project '${PROJECT_NAME}'..."

    local response=$(curl -sf "${ZITADEL_URL}/management/v1/projects" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{\"name\": \"${PROJECT_NAME}\"}" \
        2>/dev/null || echo "")

    PROJECT_ID=$(echo "$response" | jq -r '.id // empty')

    if [ -z "$PROJECT_ID" ]; then
        log_warn "Could not create project. It may already exist."
        # Try to find existing project
        local projects=$(curl -sf "${ZITADEL_URL}/management/v1/projects/_search" \
            -H "Authorization: Bearer ${ACCESS_TOKEN}" \
            -H "Content-Type: application/json" \
            -d '{"queries":[{"nameQuery":{"name":"choice-sherpa","method":"TEXT_QUERY_METHOD_EQUALS"}}]}' \
            2>/dev/null || echo "")

        PROJECT_ID=$(echo "$projects" | jq -r '.result[0].id // empty')
    fi

    if [ -z "$PROJECT_ID" ]; then
        log_error "Failed to create or find project"
        exit 1
    fi

    log_info "Project ID: ${PROJECT_ID}"
}

# =============================================================================
# Create OIDC Web Application (for frontend)
# =============================================================================
create_web_app() {
    log_info "Creating OIDC web application..."

    local response=$(curl -sf "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/oidc" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{
            "name": "choice-sherpa-frontend",
            "redirectUris": [
                "http://localhost:5173/auth/callback/zitadel",
                "http://localhost:3000/auth/callback/zitadel"
            ],
            "postLogoutRedirectUris": [
                "http://localhost:5173",
                "http://localhost:3000"
            ],
            "responseTypes": ["OIDC_RESPONSE_TYPE_CODE"],
            "grantTypes": ["OIDC_GRANT_TYPE_AUTHORIZATION_CODE", "OIDC_GRANT_TYPE_REFRESH_TOKEN"],
            "appType": "OIDC_APP_TYPE_WEB",
            "authMethodType": "OIDC_AUTH_METHOD_TYPE_BASIC",
            "accessTokenType": "OIDC_TOKEN_TYPE_JWT",
            "idTokenRoleAssertion": true,
            "idTokenUserinfoAssertion": true
        }' \
        2>/dev/null || echo "")

    FRONTEND_CLIENT_ID=$(echo "$response" | jq -r '.clientId // empty')
    FRONTEND_CLIENT_SECRET=$(echo "$response" | jq -r '.clientSecret // empty')

    if [ -z "$FRONTEND_CLIENT_ID" ]; then
        log_warn "Could not create web app. It may already exist."
    else
        log_info "Frontend Client ID: ${FRONTEND_CLIENT_ID}"
    fi
}

# =============================================================================
# Create API Application (for backend service account)
# =============================================================================
create_api_app() {
    log_info "Creating API application for backend..."

    local response=$(curl -sf "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/api" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{
            "name": "choice-sherpa-backend",
            "authMethodType": "API_AUTH_METHOD_TYPE_BASIC"
        }' \
        2>/dev/null || echo "")

    BACKEND_CLIENT_ID=$(echo "$response" | jq -r '.clientId // empty')
    BACKEND_CLIENT_SECRET=$(echo "$response" | jq -r '.clientSecret // empty')

    if [ -z "$BACKEND_CLIENT_ID" ]; then
        log_warn "Could not create API app. It may already exist."
    else
        log_info "Backend Client ID: ${BACKEND_CLIENT_ID}"
    fi
}

# =============================================================================
# Print manual setup instructions
# =============================================================================
print_manual_setup() {
    cat << 'EOF'

╔══════════════════════════════════════════════════════════════════════════════╗
║                     ZITADEL MANUAL SETUP INSTRUCTIONS                        ║
╚══════════════════════════════════════════════════════════════════════════════╝

1. Open Zitadel Console: http://localhost:8085/ui/console

2. Login with:
   - Username: admin
   - Password: Admin123!

3. Create a Project:
   - Go to "Projects" → "Create New Project"
   - Name: choice-sherpa
   - Save

4. Create Web Application (Frontend):
   - In the project, click "New" → "Web"
   - Name: choice-sherpa-frontend
   - Redirect URIs:
     - http://localhost:5173/auth/callback/zitadel
     - http://localhost:3000/auth/callback/zitadel
   - Post Logout URIs:
     - http://localhost:5173
     - http://localhost:3000
   - Auth Method: Basic
   - Save and copy the Client ID and Client Secret

5. Create API Application (Backend - optional):
   - In the project, click "New" → "API"
   - Name: choice-sherpa-backend
   - Auth Method: Basic
   - Save and copy the Client ID and Client Secret

6. Update your frontend/.env file:

   AUTH_ZITADEL_ISSUER=http://localhost:8085
   AUTH_ZITADEL_CLIENT_ID=<your-frontend-client-id>
   AUTH_ZITADEL_CLIENT_SECRET=<your-frontend-client-secret>
   AUTH_SECRET=<generate-with: openssl rand -base64 32>

EOF
}

# =============================================================================
# Print environment configuration
# =============================================================================
print_env_config() {
    cat << EOF

╔══════════════════════════════════════════════════════════════════════════════╗
║                         ENVIRONMENT CONFIGURATION                            ║
╚══════════════════════════════════════════════════════════════════════════════╝

Add these to your frontend/.env file:

# Zitadel OIDC Configuration
AUTH_ZITADEL_ISSUER=http://localhost:8085
AUTH_ZITADEL_CLIENT_ID=${FRONTEND_CLIENT_ID:-<see-zitadel-console>}
AUTH_ZITADEL_CLIENT_SECRET=${FRONTEND_CLIENT_SECRET:-<see-zitadel-console>}
AUTH_SECRET=$(openssl rand -base64 32)

# API Backend
PUBLIC_API_URL=http://localhost:8080

Add these to your backend/.env file (if using service account):

ZITADEL_ISSUER=http://localhost:8085
ZITADEL_CLIENT_ID=${BACKEND_CLIENT_ID:-<see-zitadel-console>}
ZITADEL_CLIENT_SECRET=${BACKEND_CLIENT_SECRET:-<see-zitadel-console>}
ZITADEL_AUDIENCE=choice-sherpa

═══════════════════════════════════════════════════════════════════════════════

Zitadel Console: http://localhost:8085/ui/console
Admin Login: admin / Admin123!

EOF
}

# =============================================================================
# Main
# =============================================================================
main() {
    echo ""
    echo "╔══════════════════════════════════════════════════════════════════════════════╗"
    echo "║              CHOICE SHERPA - ZITADEL OIDC SETUP                              ║"
    echo "╚══════════════════════════════════════════════════════════════════════════════╝"
    echo ""

    wait_for_zitadel

    # Try API-based setup first
    get_admin_token

    if [ -n "$ACCESS_TOKEN" ]; then
        create_project
        create_web_app
        create_api_app
        print_env_config
    fi

    log_info "Setup complete!"
    log_info "Access Zitadel Console at: ${ZITADEL_URL}/ui/console"
}

main "$@"
