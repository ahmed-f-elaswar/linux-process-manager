#!/bin/bash
# Example script: Automated process monitoring and control
# Demonstrates using the Process Manager REST API from bash

API_URL="http://localhost:8080/api"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to get system info
get_system_info() {
    echo -e "${GREEN}System Information:${NC}"
    curl -s "${API_URL}/system" | jq '.'
    echo
}

# Function to get top CPU processes
get_top_cpu() {
    echo -e "${GREEN}Top 10 CPU Consumers:${NC}"
    curl -s "${API_URL}/processes" | \
        jq -r 'sort_by(.cpu_usage) | reverse | .[:10] | 
               .[] | "\(.pid)\t\(.cpu_usage)%\t\(.name)"' | \
        column -t
    echo
}

# Function to get top memory processes
get_top_memory() {
    echo -e "${GREEN}Top 10 Memory Consumers:${NC}"
    curl -s "${API_URL}/processes" | \
        jq -r 'sort_by(.memory_usage) | reverse | .[:10] | 
               .[] | "\(.pid)\t\(.memory_usage) KB\t\(.name)"' | \
        column -t
    echo
}

# Function to find processes by name
find_process() {
    local name=$1
    echo -e "${GREEN}Processes matching '${name}':${NC}"
    curl -s "${API_URL}/processes" | \
        jq -r ".[] | select(.name | contains(\"${name}\")) | 
               \"\(.pid)\t\(.cpu_usage)%\t\(.memory_usage) KB\t\(.name)\"" | \
        column -t
    echo
}

# Function to get process history
get_process_history() {
    local pid=$1
    echo -e "${GREEN}History for PID ${pid}:${NC}"
    curl -s "${API_URL}/history/processes/${pid}" | jq '.'
    echo
}

# Function to check API health
check_health() {
    if curl -s "${API_URL}/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ API Server is running${NC}"
        return 0
    else
        echo -e "${RED}✗ API Server is not responding${NC}"
        return 1
    fi
}

# Function to monitor specific process
monitor_process() {
    local pid=$1
    echo -e "${YELLOW}Monitoring PID ${pid} (Press Ctrl+C to stop)${NC}"
    
    while true; do
        clear
        echo "=== Process Monitor ==="
        echo "PID: $pid"
        echo "Time: $(date)"
        echo
        
        curl -s "${API_URL}/processes" | \
            jq ".[] | select(.pid == ${pid})" | \
            jq -r '"Name: \(.name)
User: \(.user)
CPU: \(.cpu_usage)%
Memory: \(.memory_usage) KB (\(.memory_percent)%)
Threads: \(.threads)
Status: \(.status)"'
        
        sleep 2
    done
}

# Main menu
show_menu() {
    echo "==================================="
    echo "Process Manager API - Shell Client"
    echo "==================================="
    echo "1. System Information"
    echo "2. Top CPU Processes"
    echo "3. Top Memory Processes"
    echo "4. Find Process by Name"
    echo "5. Monitor Specific Process"
    echo "6. Process History"
    echo "7. Check API Health"
    echo "0. Exit"
    echo "==================================="
}

# Main program
main() {
    # Check if API is available
    if ! check_health; then
        echo "Please start the Process Manager API server first:"
        echo "  ./process-manager --api"
        exit 1
    fi
    
    echo
    
    while true; do
        show_menu
        read -p "Select option: " choice
        echo
        
        case $choice in
            1) get_system_info ;;
            2) get_top_cpu ;;
            3) get_top_memory ;;
            4) 
                read -p "Enter process name: " name
                find_process "$name"
                ;;
            5)
                read -p "Enter PID to monitor: " pid
                monitor_process "$pid"
                ;;
            6)
                read -p "Enter PID: " pid
                get_process_history "$pid"
                ;;
            7) check_health ;;
            0) 
                echo "Goodbye!"
                exit 0
                ;;
            *)
                echo -e "${RED}Invalid option${NC}"
                ;;
        esac
        
        read -p "Press Enter to continue..."
        clear
    done
}

# Check dependencies
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed."
    echo "Install with: sudo apt-get install jq"
    exit 1
fi

if ! command -v curl &> /dev/null; then
    echo "Error: curl is required but not installed."
    echo "Install with: sudo apt-get install curl"
    exit 1
fi

# Run main program
main
