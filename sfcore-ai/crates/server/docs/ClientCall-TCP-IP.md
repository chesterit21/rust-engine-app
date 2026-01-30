## 🚀 TCP Transport - Universal Cross-Platform

### 📁 File 3: `server_config.toml` (UPDATED - TCP Config)

```toml
# === Transport Mode ===
transport = "tcp"  # <-- Change to TCP

# === Transport Configurations ===

[tcp]
# Local only (recommended for security)
host = "127.0.0.1"  
port = 8765

# Or expose to network (⚠️ WARNING: No encryption!)
# host = "0.0.0.0"
# port = 8765
```

***

## 🧪 Testing

### **1. Start TCP Server**

```bash
cargo run --release -p sfcore-ai-server -- --transport tcp
```

Output:

```
[INFO] TCP listening on 127.0.0.1:8765 (cross-platform mode)
[INFO] Compatible with: Linux, Windows, macOS, Android, iOS, etc.
```

***

## 📨 Template Chat via TCP - Format & Examples

Oke bro, TCP pakai **JSON format** yang sama kayak UDS. Ada 2 cara:

***

## 🎯 Format Request

### **Option 1: Raw Prompt** (tanpa template)

```json
{
  "prompt": "Hello, how are you?",
  "stream": true,
  "max_tokens": 100
}
```

### **Option 2: Messages Array** (auto apply template)

```json
{
  "messages": [
    {"role": "system", "content": "You are a helpful assistant"},
    {"role": "user", "content": "Explain quantum computing"}
  ],
  "stream": true,
  "max_tokens": 200
}
```

***

## 🧪 Testing - Berbagai Client

### **1. Netcat (Linux/macOS)**

```bash
# One-liner with messages
echo '{
  "messages": [
    {"role": "system", "content": "You are a coding expert"},
    {"role": "user", "content": "Write hello world in Rust"}
  ],
  "stream": true,
  "max_tokens": 150
}' | nc 127.0.0.1 8765
```

**Compact version:**

```bash
echo '{"messages":[{"role":"user","content":"Hello"}],"stream":true,"max_tokens":50}' | nc 127.0.0.1 8765
```

***

### **2. Python Client (Full Featured)**

```python
#!/usr/bin/env python3
import socket
import json

def tcp_chat(messages, stream=True, max_tokens=200, host="127.0.0.1", port=8765):
    """
    Send chat messages to TCP server
    
    messages: List of {"role": "...", "content": "..."}
    """
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect((host, port))
    
    request = {
        "messages": messages,
        "stream": stream,
        "max_tokens": max_tokens
    }
    
    # Send request (newline terminated)
    request_json = json.dumps(request) + "\n"
    sock.sendall(request_json.encode('utf-8'))
    
    # Receive response
    buffer = ""
    output = ""
    
    try:
        while True:
            chunk = sock.recv(4096).decode('utf-8')
            if not chunk:
                break
            
            buffer += chunk
            
            # Process line by line
            while "\n" in buffer:
                line, buffer = buffer.split("\n", 1)
                if not line.strip():
                    continue
                
                response = json.loads(line)
                
                # Streaming token
                if "token" in response:
                    token = response["token"]
                    print(token, end="", flush=True)
                    output += token
                
                # Final response
                elif "done" in response and response["done"]:
                    print("\n")
                    metrics = response.get("metrics", {})
                    print(f"\n[Metrics]")
                    print(f"  Tokens: {metrics.get('tokens_generated', 0)}")
                    print(f"  Speed: {metrics.get('speed_tokens_sec', 0):.2f} tok/s")
                    print(f"  Time: {metrics.get('total_time_ms', 0)} ms")
                    sock.close()
                    return output
                
                # Error
                elif "error" in response:
                    print(f"\n[Error] {response['error']}")
                    sock.close()
                    return None
    
    finally:
        sock.close()
    
    return output


# === Example Usage ===

# Simple chat
tcp_chat([
    {"role": "user", "content": "What is Rust?"}
])

# With system prompt
tcp_chat([
    {"role": "system", "content": "You are a helpful coding assistant"},
    {"role": "user", "content": "Write a Fibonacci function in Rust"}
])

# Multi-turn conversation
tcp_chat([
    {"role": "system", "content": "You are a pirate"},
    {"role": "user", "content": "Tell me about treasure"},
    {"role": "assistant", "content": "Arrr! Treasure be the heart of every pirate's journey!"},
    {"role": "user", "content": "Where can I find it?"}
])
```

***

### **3. Rust Client**

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    messages: Vec<Message>,
    stream: bool,
    max_tokens: i32,
}

#[derive(Deserialize)]
struct StreamChunk {
    token: Option<String>,
    done: Option<bool>,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8765").await?;
    
    let request = ChatRequest {
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "Explain Rust ownership".to_string(),
            },
        ],
        stream: true,
        max_tokens: 200,
    };
    
    // Send request
    let json = serde_json::to_string(&request)?;
    stream.write_all(format!("{}\n", json).as_bytes()).await?;
    stream.flush().await?;
    
    // Receive streaming response
    let (reader, _writer) = stream.split();
    let mut lines = BufReader::new(reader).lines();
    
    while let Some(line) = lines.next_line().await? {
        let response: StreamChunk = serde_json::from_str(&line)?;
        
        if let Some(token) = response.token {
            print!("{}", token);
            std::io::Write::flush(&mut std::io::stdout())?;
        } else if response.done.unwrap_or(false) {
            println!("\n\nDone!");
            break;
        } else if let Some(err) = response.error {
            eprintln!("\nError: {}", err);
            break;
        }
    }
    
    Ok(())
}
```

***

### **4. cURL (HTTP-like simulation)**

Simpan request ke file:

**`request.json`:**

```json
{
  "messages": [
    {"role": "system", "content": "You are an expert programmer"},
    {"role": "user", "content": "Write bubble sort in Rust"}
  ],
  "stream": false,
  "max_tokens": 300
}
```

**Send via netcat:**

```bash
cat request.json | nc 127.0.0.1 8765
```

***

### **5. JavaScript/Node.js Client**

```javascript
const net = require('net');

function tcpChat(messages, options = {}) {
    const {
        stream = true,
        maxTokens = 200,
        host = '127.0.0.1',
        port = 8765
    } = options;
    
    return new Promise((resolve, reject) => {
        const client = net.createConnection({ host, port }, () => {
            const request = JSON.stringify({
                messages,
                stream,
                max_tokens: maxTokens
            }) + '\n';
            
            client.write(request);
        });
        
        let buffer = '';
        let output = '';
        
        client.on('data', (data) => {
            buffer += data.toString();
            
            // Process line by line
            let lines = buffer.split('\n');
            buffer = lines.pop(); // Keep incomplete line
            
            lines.forEach(line => {
                if (!line.trim()) return;
                
                try {
                    const response = JSON.parse(line);
                    
                    if (response.token) {
                        process.stdout.write(response.token);
                        output += response.token;
                    } else if (response.done) {
                        console.log('\n\n[Metrics]', response.metrics);
                        client.end();
                        resolve(output);
                    } else if (response.error) {
                        console.error('\n[Error]', response.error);
                        client.end();
                        reject(new Error(response.error));
                    }
                } catch (e) {
                    console.error('JSON parse error:', e);
                }
            });
        });
        
        client.on('error', reject);
        client.on('end', () => resolve(output));
    });
}

// Usage
tcpChat([
    { role: 'system', content: 'You are a helpful assistant' },
    { role: 'user', content: 'What is async/await?' }
]).then(output => {
    console.log('\n[Full Output]', output);
}).catch(console.error);
```

***

## 🎨 Template Support (Auto-Detected)

Server **otomatis detect** template dari model file:

### **ChatML (Qwen, Phi, etc.)**

```
<|im_start|>system
You are a helpful assistant<|im_end|>
<|im_start|>user
Hello<|im_end|>
<|im_start|>assistant
```

### **Llama 3**

```
<|begin_of_text|><|start_header_id|>system<|end_header_id|>
You are a helpful assistant<|eot_id|>
<|start_header_id|>user<|end_header_id|>
Hello<|eot_id|>
<|start_header_id|>assistant<|end_header_id|>
```

### **Gemma**

```
<bos><start_of_turn>user
Hello<end_of_turn>
<start_of_turn>model
```

**Client gak perlu tahu template format! Server handle semua!** ✅

***

## 📋 Complete Example - Multi-Turn Chat

```python
import socket
import json

def chat_session(host="127.0.0.1", port=8765):
    """Interactive chat session"""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect((host, port))
    
    conversation = [
        {"role": "system", "content": "You are a helpful AI assistant"}
    ]
    
    print("Chat started! Type 'quit' to exit.\n")
    
    while True:
        user_input = input("You: ")
        if user_input.lower() == 'quit':
            break
        
        # Add user message
        conversation.append({"role": "user", "content": user_input})
        
        # Send request
        request = {
            "messages": conversation,
            "stream": True,
            "max_tokens": 300
        }
        sock.sendall((json.dumps(request) + "\n").encode())
        
        # Receive response
        print("AI: ", end="", flush=True)
        buffer = ""
        assistant_response = ""
        
        while True:
            chunk = sock.recv(4096).decode()
            if not chunk:
                break
            
            buffer += chunk
            while "\n" in buffer:
                line, buffer = buffer.split("\n", 1)
                if not line:
                    continue
                
                resp = json.loads(line)
                
                if "token" in resp:
                    print(resp["token"], end="", flush=True)
                    assistant_response += resp["token"]
                elif "done" in resp:
                    print("\n")
                    break
            
            if "done" in resp:
                break
        
        # Add assistant response to conversation
        conversation.append({"role": "assistant", "content": assistant_response})
    
    sock.close()

# Run
chat_session()
```

***

## ✅ Summary

**Format:**

- ✅ `messages` array → Auto apply template
- ✅ `prompt` string → Raw input (no template)

**Clients:**

- ✅ Netcat (quick test)
- ✅ Python (production)
- ✅ Rust (native)
- ✅ Node.js (web backend)
- ✅ Any language with TCP socket support

**Template:**

- ✅ Auto-detected dari model
- ✅ Support ChatML, Llama, Gemma, dll
- ✅ Client gak perlu tahu format!

Mau test sekarang? 🚀

## 🌐 Remote Access (Network)

### **Option 1: Direct (⚠️ Insecure)**

```toml
[tcp]
host = "0.0.0.0"  # Listen on all interfaces
port = 8765
```

Client connect:

```bash
echo '{"prompt": "test"}' | nc 192.168.1.100 8765
```

**⚠️ WARNING: No encryption! Don't use over internet!**

***

### **Option 2: SSH Tunnel (✅ Recommended)**

Server (local only):

```toml
[tcp]
host = "127.0.0.1"
port = 8765
```

Client (remote):

```bash
# Create SSH tunnel
ssh -L 8765:localhost:8765 user@server-ip

# Connect via localhost (encrypted via SSH)
echo '{"prompt": "test"}' | nc 127.0.0.1 8765
```

***

### **Option 3: VPN/Tailscale**

Use Tailscale/WireGuard untuk private network, bind ke Tailscale IP.

***

## 📊 Comparison Table

| Transport | Platform | Remote | Encryption | Use Case |
|-----------|----------|--------|------------|----------|
| **UDS** | Unix only | ❌ Local | N/A | Linux/Mac local IPC |
| **TCP** | All OS ✅ | ✅ Yes | ❌ No | Cross-platform, LAN |
| **HTTP-SSE** | All OS ✅ | ✅ Yes | ✅ HTTPS | Web clients, API |

***

## ✅ Summary

**TCP Transport Features:**

- ✅ **Universal**: Linux, Windows, macOS, Android, iOS, embedded
- ✅ **Protocol**: JSON newline (same as UDS)
- ✅ **Streaming**: Full support
- ✅ **Persistent**: Keep-alive connections
- ✅ **Performance**: Near-native (minimal overhead)
- ⚠️ **Security**: No encryption (use SSH tunnel or local only)

**Best Practices:**

1. **Local dev**: `host = "127.0.0.1"`
2. **LAN**: `host = "0.0.0.0"` + firewall
3. **Remote**: SSH tunnel or VPN
4. **Production**: Use HTTP-SSE with HTTPS instead

**Android/iOS:** TCP work perfectly, tinggal connect via socket library! 🚀
