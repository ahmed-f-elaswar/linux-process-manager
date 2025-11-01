#!/usr/bin/env python3
"""
Example script: Export process metrics to CSV

This script fetches process data from the API and exports it to CSV format
for analysis in spreadsheet applications.
"""

import requests
import csv
from datetime import datetime

API_URL = "http://localhost:8080/api"
OUTPUT_FILE = "process_snapshot.csv"

def export_to_csv():
    """Export current process snapshot to CSV"""
    print(f"Fetching process data from {API_URL}...")
    
    try:
        response = requests.get(f"{API_URL}/processes")
        response.raise_for_status()
        processes = response.json()
        
        print(f"Retrieved {len(processes)} processes")
        
        # Define CSV columns
        fieldnames = [
            'pid', 'ppid', 'name', 'user', 'cpu_usage', 
            'memory_usage', 'memory_percent', 'status', 
            'threads', 'network_connections', 'is_container',
            'container_id', 'gpu_memory', 'command'
        ]
        
        # Write to CSV
        with open(OUTPUT_FILE, 'w', newline='') as csvfile:
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            
            for proc in processes:
                # Create row with only the fields we want
                row = {field: proc.get(field, '') for field in fieldnames}
                writer.writerow(row)
        
        print(f"✓ Data exported to {OUTPUT_FILE}")
        print(f"  Timestamp: {datetime.now()}")
        
    except requests.exceptions.RequestException as e:
        print(f"✗ Error: {e}")
    except IOError as e:
        print(f"✗ File error: {e}")

if __name__ == "__main__":
    export_to_csv()
