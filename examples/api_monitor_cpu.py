#!/usr/bin/env python3
"""
Example script: Monitor high CPU processes via REST API

This script demonstrates how to use the Process Manager REST API
to monitor and alert on high CPU usage.
"""

import requests
import time
import json
from datetime import datetime

# Configuration
API_URL = "http://localhost:8080/api"
CPU_THRESHOLD = 50.0  # Alert if CPU usage exceeds this percentage
CHECK_INTERVAL = 5    # Check every N seconds

def get_all_processes():
    """Fetch all processes from the API"""
    try:
        response = requests.get(f"{API_URL}/processes")
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Error fetching processes: {e}")
        return []

def get_system_info():
    """Fetch system information"""
    try:
        response = requests.get(f"{API_URL}/system")
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Error fetching system info: {e}")
        return {}

def alert_high_cpu(process):
    """Alert on high CPU usage"""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"[{timestamp}] ⚠️  HIGH CPU ALERT!")
    print(f"  Process: {process['name']} (PID: {process['pid']})")
    print(f"  CPU Usage: {process['cpu_usage']:.1f}%")
    print(f"  Memory: {process['memory_usage']} KB")
    print(f"  User: {process['user']}")
    print()

def main():
    print("Process Manager API - High CPU Monitor")
    print(f"Monitoring for CPU usage > {CPU_THRESHOLD}%")
    print(f"Checking every {CHECK_INTERVAL} seconds")
    print("Press Ctrl+C to stop\n")
    
    try:
        while True:
            processes = get_all_processes()
            
            if processes:
                high_cpu_processes = [
                    p for p in processes 
                    if p['cpu_usage'] > CPU_THRESHOLD
                ]
                
                if high_cpu_processes:
                    # Sort by CPU usage (highest first)
                    high_cpu_processes.sort(
                        key=lambda x: x['cpu_usage'], 
                        reverse=True
                    )
                    
                    for proc in high_cpu_processes[:5]:  # Top 5
                        alert_high_cpu(proc)
                else:
                    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                    print(f"[{timestamp}] ✓ All processes within normal CPU range")
            
            time.sleep(CHECK_INTERVAL)
            
    except KeyboardInterrupt:
        print("\n\nMonitoring stopped.")

if __name__ == "__main__":
    main()
