#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
#  Noesis API Explorer — Interactive Terminal Client
#  A rich CLI for exploring the Tryambakam Noesis API
# ═══════════════════════════════════════════════════════════════

set -euo pipefail

# ── Colors & Formatting ──────────────────────────────────────
BOLD="\033[1m"
DIM="\033[2m"
RESET="\033[0m"
CYAN="\033[36m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
MAGENTA="\033[35m"
BLUE="\033[34m"
WHITE="\033[37m"

# ── Config ────────────────────────────────────────────────────
DEFAULT_URL="https://selemene.tryambakam.space"
BASE_URL="${NOESIS_URL:-$DEFAULT_URL}"
API_KEY="${NOESIS_API_KEY:-}"

# ── Helpers ───────────────────────────────────────────────────
print_header() {
    echo ""
    echo -e "${MAGENTA}${BOLD}"
    echo "  ════════════════════════════════════════════════════"
    echo "   ◈  त्र्यम्बकम्  ◈  NOESIS API EXPLORER"
    echo "  ════════════════════════════════════════════════════"
    echo -e "${RESET}"
    echo -e "  ${DIM}Base URL: ${BASE_URL}${RESET}"
    echo ""
}

print_divider() {
    echo -e "${DIM}  ────────────────────────────────────────────────${RESET}"
}

print_success() {
    echo -e "  ${GREEN}${BOLD}$1${RESET}"
}

print_error() {
    echo -e "  ${RED}${BOLD}$1${RESET}"
}

print_info() {
    echo -e "  ${CYAN}$1${RESET}"
}

print_label() {
    echo -e "  ${YELLOW}${BOLD}$1${RESET}"
}

json_format() {
    if command -v jq &>/dev/null; then
        jq '.' 2>/dev/null || cat
    elif command -v python3 &>/dev/null; then
        python3 -m json.tool 2>/dev/null || cat
    else
        cat
    fi
}

json_format_colored() {
    if command -v jq &>/dev/null; then
        jq -C '.' 2>/dev/null || json_format
    else
        json_format
    fi
}

api_call() {
    local method="$1"
    local path="$2"
    local data="${3:-}"
    local url="${BASE_URL}${path}"

    local args=(-s -w "\n___HTTP_STATUS:%{http_code}___")
    args+=(-H "X-API-Key: ${API_KEY}")

    if [[ "$method" == "POST" ]]; then
        args+=(-X POST -H "Content-Type: application/json")
        if [[ -n "$data" ]]; then
            args+=(-d "$data")
        fi
    fi

    local response
    response=$(curl "${args[@]}" "$url" 2>&1)

    local status
    status=$(echo "$response" | grep -o '___HTTP_STATUS:[0-9]*___' | grep -o '[0-9]*')
    local body
    body=$(echo "$response" | sed 's/___HTTP_STATUS:[0-9]*___//')

    if [[ "$status" -ge 200 && "$status" -lt 300 ]]; then
        echo -e "  ${GREEN}${BOLD}HTTP $status${RESET}"
    elif [[ "$status" -ge 400 ]]; then
        echo -e "  ${RED}${BOLD}HTTP $status${RESET}"
    else
        echo -e "  ${YELLOW}${BOLD}HTTP $status${RESET}"
    fi

    echo ""
    echo "$body" | json_format_colored | sed 's/^/    /'
    echo ""
}

# ── Prompt for API key if not set ─────────────────────────────
ensure_api_key() {
    if [[ -z "$API_KEY" ]]; then
        echo ""
        print_error "No API key found."
        echo ""
        print_info "Set it with:  export NOESIS_API_KEY=\"nk_your_key_here\""
        print_info "Or enter it now:"
        echo ""
        echo -en "  ${BOLD}API Key: ${RESET}"
        read -r API_KEY
        if [[ -z "$API_KEY" ]]; then
            print_error "Cannot continue without an API key."
            exit 1
        fi
        echo ""
        print_success "API key set for this session."
    fi
}

# ── Menu Actions ──────────────────────────────────────────────
action_health() {
    print_label "Health Check"
    print_divider
    echo ""
    print_info "GET /health/live"
    local response
    response=$(curl -s "${BASE_URL}/health/live" 2>&1)
    echo ""
    echo "$response" | json_format_colored | sed 's/^/    /'
    echo ""

    print_info "GET /health/ready"
    response=$(curl -s "${BASE_URL}/health/ready" 2>&1)
    echo ""
    echo "$response" | json_format_colored | sed 's/^/    /'
    echo ""
}

action_list_engines() {
    print_label "Available Engines"
    print_divider
    echo ""
    print_info "GET /api/v1/engines"
    api_call GET "/api/v1/engines"
}

action_list_workflows() {
    print_label "Available Workflows"
    print_divider
    echo ""
    print_info "GET /api/v1/workflows"
    api_call GET "/api/v1/workflows"
}

action_engine_info() {
    print_label "Engine Info"
    print_divider
    echo ""
    echo -e "  ${DIM}Engines: biofield, biorhythm, gene-keys, human-design,"
    echo -e "           numerology, panchanga, vedic-clock, vimshottari${RESET}"
    echo ""
    echo -en "  ${BOLD}Engine ID: ${RESET}"
    read -r engine_id
    if [[ -z "$engine_id" ]]; then return; fi
    echo ""
    print_info "GET /api/v1/engines/${engine_id}/info"
    api_call GET "/api/v1/engines/${engine_id}/info"
}

read_birth_data() {
    echo ""
    print_info "Enter birth data (press Enter for defaults):"
    echo ""

    echo -en "  ${BOLD}Date ${DIM}[1991-08-13]${RESET}${BOLD}: ${RESET}"
    read -r bd_date
    bd_date="${bd_date:-1991-08-13}"

    echo -en "  ${BOLD}Time ${DIM}[13:31]${RESET}${BOLD}: ${RESET}"
    read -r bd_time
    bd_time="${bd_time:-13:31}"

    echo -en "  ${BOLD}Name ${DIM}[optional]${RESET}${BOLD}: ${RESET}"
    read -r bd_name

    echo -en "  ${BOLD}Latitude ${DIM}[12.9716]${RESET}${BOLD}: ${RESET}"
    read -r bd_lat
    bd_lat="${bd_lat:-12.9716}"

    echo -en "  ${BOLD}Longitude ${DIM}[77.5946]${RESET}${BOLD}: ${RESET}"
    read -r bd_lng
    bd_lng="${bd_lng:-77.5946}"

    echo -en "  ${BOLD}Timezone ${DIM}[Asia/Kolkata]${RESET}${BOLD}: ${RESET}"
    read -r bd_tz
    bd_tz="${bd_tz:-Asia/Kolkata}"

    # Build JSON
    local name_field=""
    if [[ -n "$bd_name" ]]; then
        name_field="\"name\":\"${bd_name}\","
    fi

    BIRTH_DATA_JSON="{${name_field}\"date\":\"${bd_date}\",\"time\":\"${bd_time}\",\"latitude\":${bd_lat},\"longitude\":${bd_lng},\"timezone\":\"${bd_tz}\"}"
}

action_calculate() {
    print_label "Run Engine Calculation"
    print_divider
    echo ""
    echo -e "  ${DIM}Engines: biofield, biorhythm, gene-keys, human-design,"
    echo -e "           numerology, panchanga, vedic-clock, vimshottari${RESET}"
    echo ""
    echo -en "  ${BOLD}Engine ID: ${RESET}"
    read -r engine_id
    if [[ -z "$engine_id" ]]; then return; fi

    read_birth_data

    local payload="{\"birth_data\":${BIRTH_DATA_JSON}}"

    echo ""
    print_info "POST /api/v1/engines/${engine_id}/calculate"
    echo -e "  ${DIM}Payload: ${payload}${RESET}"
    api_call POST "/api/v1/engines/${engine_id}/calculate" "$payload"
}

action_workflow() {
    print_label "Execute Workflow"
    print_divider
    echo ""
    echo -e "  ${DIM}Workflows: birth-blueprint, daily-practice, decision-support,"
    echo -e "             self-inquiry, creative-expression, full-spectrum${RESET}"
    echo ""
    echo -en "  ${BOLD}Workflow ID: ${RESET}"
    read -r workflow_id
    if [[ -z "$workflow_id" ]]; then return; fi

    read_birth_data

    local payload="{\"birth_data\":${BIRTH_DATA_JSON}}"

    echo ""
    print_info "POST /api/v1/workflows/${workflow_id}/execute"
    api_call POST "/api/v1/workflows/${workflow_id}/execute" "$payload"
}

action_quick_test() {
    print_label "Quick Test — All Engines"
    print_divider
    echo ""
    print_info "Running a quick calculation on each engine..."

    local birth='{"birth_data":{"name":"QuickTest","date":"1991-08-13","time":"13:31","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"}}'
    local engines=("numerology" "biorhythm" "panchanga" "vedic-clock" "human-design" "gene-keys" "vimshottari" "biofield")

    for engine in "${engines[@]}"; do
        echo ""
        echo -en "  ${CYAN}${BOLD}${engine}${RESET} ... "

        local start_ms
        start_ms=$(python3 -c 'import time; print(int(time.time()*1000))')

        local response
        response=$(curl -s -w "___HTTP_STATUS:%{http_code}___" \
            -X POST "${BASE_URL}/api/v1/engines/${engine}/calculate" \
            -H "X-API-Key: ${API_KEY}" \
            -H "Content-Type: application/json" \
            -d "$birth" 2>&1)

        local end_ms
        end_ms=$(python3 -c 'import time; print(int(time.time()*1000))')

        local elapsed=$(( end_ms - start_ms ))
        local status
        status=$(echo "$response" | grep -o '___HTTP_STATUS:[0-9]*___' | grep -o '[0-9]*')

        if [[ "$status" == "200" ]]; then
            echo -e "${GREEN}${BOLD}OK${RESET} ${DIM}(${elapsed}ms)${RESET}"
        else
            echo -e "${RED}${BOLD}FAIL (HTTP ${status})${RESET} ${DIM}(${elapsed}ms)${RESET}"
        fi
    done
    echo ""
}

action_swagger() {
    local url="${BASE_URL}/api/docs"
    print_label "Opening Swagger UI"
    print_divider
    echo ""
    print_info "URL: ${url}"
    echo ""

    if command -v open &>/dev/null; then
        open "$url"
        print_success "Opened in browser."
    elif command -v xdg-open &>/dev/null; then
        xdg-open "$url"
        print_success "Opened in browser."
    else
        print_info "Open this URL in your browser: ${url}"
    fi
    echo ""
}

# ── Main Menu ─────────────────────────────────────────────────
main_menu() {
    while true; do
        echo ""
        echo -e "  ${BOLD}What would you like to do?${RESET}"
        echo ""
        echo -e "  ${WHITE}${BOLD}1${RESET}  ${CYAN}Health Check${RESET}         ${DIM}— Is the API alive?${RESET}"
        echo -e "  ${WHITE}${BOLD}2${RESET}  ${CYAN}List Engines${RESET}         ${DIM}— See available engines${RESET}"
        echo -e "  ${WHITE}${BOLD}3${RESET}  ${CYAN}List Workflows${RESET}       ${DIM}— See multi-engine workflows${RESET}"
        echo -e "  ${WHITE}${BOLD}4${RESET}  ${CYAN}Engine Info${RESET}          ${DIM}— Details about an engine${RESET}"
        echo -e "  ${WHITE}${BOLD}5${RESET}  ${GREEN}Run Calculation${RESET}      ${DIM}— Call an engine with birth data${RESET}"
        echo -e "  ${WHITE}${BOLD}6${RESET}  ${GREEN}Execute Workflow${RESET}     ${DIM}— Run a multi-engine workflow${RESET}"
        echo -e "  ${WHITE}${BOLD}7${RESET}  ${YELLOW}Quick Test All${RESET}       ${DIM}— Hit every engine, see status${RESET}"
        echo -e "  ${WHITE}${BOLD}8${RESET}  ${MAGENTA}Open Swagger UI${RESET}     ${DIM}— Interactive API docs in browser${RESET}"
        echo -e "  ${WHITE}${BOLD}q${RESET}  ${DIM}Quit${RESET}"
        echo ""
        echo -en "  ${BOLD}> ${RESET}"
        read -r choice

        case "$choice" in
            1) action_health ;;
            2) action_list_engines ;;
            3) action_list_workflows ;;
            4) action_engine_info ;;
            5) action_calculate ;;
            6) action_workflow ;;
            7) action_quick_test ;;
            8) action_swagger ;;
            q|Q|quit|exit) echo ""; print_info "Namaste."; echo ""; break ;;
            *) print_error "Unknown option: $choice" ;;
        esac
    done
}

# ── Entry Point ───────────────────────────────────────────────
print_header
ensure_api_key
main_menu
