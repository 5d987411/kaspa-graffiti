#!/usr/bin/env python3
"""
Kaspa Graffiti Test Server
Proxies CLI commands from the browser test suite
"""

from flask import Flask, request, jsonify, send_from_directory
from flask_cors import CORS
import subprocess
import json
import os
from pathlib import Path

app = Flask(__name__)
CORS(app)

# Configuration
CLI_PATH = Path(__file__).parent.parent / "target" / "release" / "kaspa-graffiti-cli"
TESTS_DIR = Path(__file__).parent

@app.route('/')
def index():
    return send_from_directory(TESTS_DIR, 'index.html')

@app.route('/<path:filename>')
def serve_file(filename):
    return send_from_directory(TESTS_DIR, filename)

@app.route('/api/cli/generate', methods=['POST'])
def cli_generate():
    """Generate a new wallet using CLI"""
    try:
        result = subprocess.run(
            [str(CLI_PATH), 'generate'],
            capture_output=True,
            text=True,
            timeout=10
        )
        if result.returncode == 0:
            return jsonify({
                'success': True,
                'output': result.stdout,
                'data': json.loads(result.stdout)
            })
        else:
            return jsonify({
                'success': False,
                'error': result.stderr
            }), 500
    except Exception as e:
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/cli/load', methods=['POST'])
def cli_load():
    """Load wallet from private key using CLI"""
    data = request.get_json()
    private_key = data.get('private_key', '')
    
    if not private_key:
        return jsonify({
            'success': False,
            'error': 'Private key is required'
        }), 400
    
    try:
        result = subprocess.run(
            [str(CLI_PATH), 'load', private_key],
            capture_output=True,
            text=True,
            timeout=10
        )
        if result.returncode == 0:
            return jsonify({
                'success': True,
                'output': result.stdout,
                'data': json.loads(result.stdout)
            })
        else:
            return jsonify({
                'success': False,
                'error': result.stderr
            }), 500
    except Exception as e:
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/cli/balance', methods=['POST'])
def cli_balance():
    """Get balance using CLI"""
    data = request.get_json()
    address = data.get('address', '')
    
    try:
        result = subprocess.run(
            [str(CLI_PATH), 'balance', address],
            capture_output=True,
            text=True,
            timeout=10
        )
        if result.returncode == 0:
            return jsonify({
                'success': True,
                'output': result.stdout,
                'data': json.loads(result.stdout)
            })
        else:
            return jsonify({
                'success': False,
                'error': result.stderr
            }), 500
    except Exception as e:
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/cli/graffiti', methods=['POST'])
def cli_graffiti():
    """Send graffiti using CLI"""
    data = request.get_json()
    private_key = data.get('private_key', '')
    message = data.get('message', '')
    mimetype = data.get('mimetype', 'text/plain')
    fee_rate = data.get('fee_rate', 1000)
    
    try:
        result = subprocess.run(
            [
                str(CLI_PATH), 'graffiti', 
                private_key, 
                message,
                mimetype,
                str(fee_rate)
            ],
            capture_output=True,
            text=True,
            timeout=30
        )
        if result.returncode == 0:
            return jsonify({
                'success': True,
                'output': result.stdout,
                'data': json.loads(result.stdout)
            })
        else:
            return jsonify({
                'success': False,
                'error': result.stderr
            }), 500
    except Exception as e:
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/cli/utxos', methods=['POST'])
def cli_utxos():
    """Get UTXOs using CLI"""
    data = request.get_json()
    address = data.get('address', '')
    
    try:
        result = subprocess.run(
            [str(CLI_PATH), 'utxos', address],
            capture_output=True,
            text=True,
            timeout=10
        )
        if result.returncode == 0:
            return jsonify({
                'success': True,
                'output': result.stdout,
                'data': json.loads(result.stdout)
            })
        else:
            return jsonify({
                'success': False,
                'error': result.stderr
            }), 500
    except Exception as e:
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

if __name__ == '__main__':
    print(f"CLI Path: {CLI_PATH}")
    print(f"Tests Dir: {TESTS_DIR}")
    print("Starting server on http://localhost:8765")
    print("\nAvailable endpoints:")
    print("  GET  /                    - Test suite")
    print("  POST /api/cli/generate    - Generate wallet")
    print("  POST /api/cli/balance     - Check balance")
    print("  POST /api/cli/utxos       - Get UTXOs")
    print("  POST /api/cli/graffiti    - Send graffiti")
    print("\nPress Ctrl+C to stop")
    app.run(host='0.0.0.0', port=8765, debug=True)
