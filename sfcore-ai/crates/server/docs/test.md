# 🚀 Full Implementation - HTTP-SSE Transport

## ✅ Testing

### **1. Start Server (HTTP-SSE mode)**

```bash
cd sfcore-ai
cargo run --release -p sfcore-ai-server
```

### **2. Test Non-Streaming**

```bash
curl -X POST http://localhost:8080/v1/inference \
  -H "Authorization: Bearer sk-sfcore-1234567890abcdef" \
  -H "X-Client-App: MyApp-v1.0" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Hello, how are you?",
    "stream": false,
    "max_tokens": 50
  }'
```

### **3. Test Streaming (SSE)**

```bash
curl -N -X POST http://localhost:8080/v1/inference \
  -H "Authorization: Bearer sk-sfcore-1234567890abcdef" \
  -H "X-Client-App: MyApp-v1.0" \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [{"role": "system", "content": "Kamu adalah AI Asisten yang dapat membantu pengguna dengan berbagai tugas dan ahli dalam bidang bahasa pemrograman."},{"role": "user", "content": "Halo bro, rust sama Go mana yang lebih cepat? dari sisi performance lebih bagus mana?"}],
    "stream": true,
    "max_tokens": 2024
  }'
```

### **4. Test Auth Failures**

```bash
# Wrong API key
curl -X POST http://localhost:8080/v1/inference \
  -H "Authorization: Bearer wrong-key" \
  -H "X-Client-App: MyApp-v1.0" \
  -H "Content-Type: application/json" \
  -d '{"prompt": "test", "stream": false}'

# Wrong client app
curl -X POST http://localhost:8080/v1/inference \
  -H "Authorization: Bearer sk-sfcore-1234567890abcdef" \
  -H "X-Client-App: UnknownApp" \
  -H "Content-Type: application/json" \
  -d '{"prompt": "test", "stream": false}'
```

***
