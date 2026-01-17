# User Manual - sfcore-cache (localcached)

**sfcore-cache** (`localcached`) adalah sistem cache lokal berkecepatan tinggi yang dirancang untuk komunikasi antar-proses (IPC) menggunakan Unix Domain Sockets (UDS). Sistem ini difokuskan untuk meminimalisir latensi dan penggunaan sumber daya dengan pendekatan "Zero-Parse" di sisi server.

## 🚀 Instalasi & Menjalankan Server

### 1. Build Server

Pastikan Anda sudah menginstall Rust. Masuk ke direktori workspace `root-app/sfcore-ai` dan jalankan:

```bash
cargo build --release -p localcached-server
```

Binary akan tersedia di `target/release/localcached-server`.

### 2. Menjalankan Server & Konfigurasi

Anda bisa menggunakan tool **CLI** (`localcached-cli`) untuk manajemen mudah, atau menggunakan **Systemd** untuk production.

#### Opsi A: Menggunakan CLI (Rekomendasi untuk Development)

Gunakan `localcached-cli` untuk menyalakan, mematikan, dan memonitor server dalam satu pintu.

```bash
# Start Server
cargo run --release -p localcached-cli -- start

# Monitor (TUI)
cargo run --release -p localcached-cli

# Stop Server
cargo run --release -p localcached-cli -- stop
```

Untuk detail lengkap penggunaan CLI, baca: [**User Manual - CLI**](./UserManual-Cli.md).

Konfigurasi tetap menggunakan Environment Variable:

```bash
export LOCALCACHED_PRESSURE_HOT=0.90
localcached-cli start
```

#### Opsi B: Menggunakan Systemd (Production)

Untuk production, jangan jalankan manual. Buatlah service file agar server otomatis berjalan dan konfigurasi tersimpan rapi.

1. **Buat file service**: `sudo nano /etc/systemd/system/localcached.service`
2. **Isi file dengan konfigurasi berikut**:

    ```ini
    [Unit]
    Description=SFCore Local Cache Daemon
    After=network.target

    [Service]
    Type=simple
    User=sfcore  # Sesuaikan dengan user linux Anda
    
    # --- KONFIGURASI DI SINI ---
    # Lokasi file socket
    Environment="LOCALCACHED_SOCKET=/run/localcached.sock"
    
    # Batas Memori (Eviction Threshold): 0.85 artinya 85%
    # Mengapa 0.85?
    # Ini adalah "Safe Zone". Saat penggunaan RAM sistem menyentuh 85%,
    # cache akan otomatis menghapus data lama (eviksi) agar sisa 15% RAM
    # tetap tersedia untuk OS dan aplikasi lain. Ini mencegah Server Crash / OOM.
    Environment="LOCALCACHED_PRESSURE_HOT=0.85"
    
    # Batas Payload Maksimal (Default 8MB)
    Environment="LOCALCACHED_MAX_FRAME=8388608"
    # ---------------------------

    ExecStart=/home/sfcore/SFCoreAIApps/AIRustTools/root-app/sfcore-ai/target/release/localcached-server
    Restart=always

    [Install]
    WantedBy=multi-user.target
    ```

3. **Reload & Start**:

    ```bash
    sudo systemctl daemon-reload
    sudo systemctl enable --now localcached
    ```

### 3. Penjelasan Konfigurasi

| Variable | Default | Penjelasan Detail |
| :--- | :--- | :--- |
| `LOCALCACHED_SOCKET` | `/run/localcached.sock` | Lokasi file Unix Socket. Pastikan user aplikasi punya akses baca/tulis ke folder ini. |
| `LOCALCACHED_MAX_FRAME` | `8388608` (8MB) | Ukuran maksimal satu item data yang bisa disimpan. Jika aplikasi mengirim data lebih besar dari ini, server akan menolak. |
| `LOCALCACHED_PRESSURE_HOT` | `0.85` | **Titik Kritis Eviksi**. Nilai `0.85` berarti server akan mulai menghapus data cache *secara agresif* ketika total RAM sistem terpakai di atas 85%. <br><br> **Kenapa penting?** Cache lokal berbagi RAM dengan OS. Tanpa batas ini, cache bisa memakan 100% RAM dan membuat OS macet (hang). Angka 0.85 adalah titik keseimbangan aman. |
| `LOCALCACHED_PUBSUB_CAP` | `256` | Kapasitas antrian pesan per topik. Jika subscriber lambat memproses >256 pesan, pesan terlama akan dibuang (dropped) agar server tidak kehabisan memori. |

---

## 💻 Panduan Implementasi Client (Smart Helper Wrappers)

Di bawah ini adalah **Smart Helper Wrappers** yang dirancang untuk kemudahan penggunaan (*Developer Experience*).

> **💡 PENTING:** Untuk panduan **Dependency Injection (DI)** dan integrasi ke Framework (Laravel, .NET, NestJS, Spring Boot, Django), silakan baca:  
> 👉 [**User Manual - Dependency Injection & Frameworks**](./UserManual-DI.md)

**Fitur Utama Helper Ini:**

1. **Auto-Serialization**: Fungsi `set()` menerima Object/Array/Class dan otomatis diubah ke JSON.
2. **Auto-Deserialization**: Fungsi `get()` otomatis mengembalikan Object asli (bukan bytes/string).
3. **Lengkap**: Mendukung `set`, `get`, `delete`, dan `subscribe` (Pub/Sub).

### 📡 Konsep & Aturan Main Pub/Sub

Fitur **Pub/Sub** (Publish/Subscribe) memungkinkan aplikasi Anda menerima notifikasi real-time saat data berubah.

**Aturan Main:**

1. **Koneksi Terblokir (Blocking)**: Saat client memanggil `subscribe()`, koneksi tersebut akan masuk ke mode "Listening". Anda **tidak bisa** mengirim perintah lain (`set`/`get`) di koneksi yang sama.
2. **Dedicated Connection**: Jika aplikasi Anda butuh mengirim data (`Set`) DAN menerima event (`Subscribe`), Anda harus membuat **dua koneksi terpisah** (dua instance client).
3. **Event Loop**: Client akan melakukan *looping* terus-menerus untuk menunggu pesan dari server.

---

### 1. 🐍 Python (Smart Wrapper)

Simpan sebagai `localcached.py`.

```python
import socket
import struct
import json
import threading

class LocalCachedClient:
    def __init__(self, socket_path="/tmp/localcached.sock"):
        self.socket_path = socket_path
        self.sock = None
        self.is_subscribed = False

    def connect(self):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(self.socket_path)

    def close(self):
        if self.sock: self.sock.close()

    def _read_exact(self, n):
        data = b''
        while len(data) < n:
            packet = self.sock.recv(n - len(data))
            if not packet: raise Exception("Connection closed")
            data += packet
        return data

    def _read_frame(self):
        # Header: [u32 len]
        len_bytes = self._read_exact(4)
        length = struct.unpack('<I', len_bytes)[0]
        # Body: [u8 status/opcode][payload]
        body = self._read_exact(length)
        return body[0], body[1:]

    def set(self, key, value, ttl_ms=0):
        """
        Menyimpan data. Value bisa berupa Dict, List, String, atau Int.
        Akan otomatis di-serialize ke JSON.
        """
        # Auto-Serialize
        if isinstance(value, (dict, list, int, float, bool)):
            val_bytes = json.dumps(value).encode('utf-8')
            fmt = 1 # JSON
        elif isinstance(value, str):
            val_bytes = value.encode('utf-8')
            fmt = 0 # Raw/String
        else:
            val_bytes = value # Assume bytes
            fmt = 0
            
        key_bytes = key.encode('utf-8')
        
        # Payload: [fmt][flags][klen][key][vlen][val][ttl]
        payload = struct.pack('<B B H', fmt, 0, len(key_bytes)) + \
                  key_bytes + \
                  struct.pack('<I', len(val_bytes)) + \
                  val_bytes + \
                  struct.pack('<Q', ttl_ms)
        
        self._send_command(0x01, payload)
        self._expect_ok()

    def get(self, key):
        """
        Mengambil data. Otomatis convert kembali ke Object asli (Dict/List).
        Return None jika tidak ditemukan.
        """
        key_bytes = key.encode('utf-8')
        payload = struct.pack('<H', len(key_bytes)) + key_bytes
        self._send_command(0x02, payload)
        
        status, body = self._read_frame()
        if status == 4: return None # NotFound
        if status != 0: raise Exception(f"Error {status}")
        
        # Body: [fmt][vlen][val][ttl]
        fmt = body[0]
        vlen = struct.unpack_from('<I', body, 1)[0]
        val_bytes = body[5 : 5+vlen]
        
        # Auto-Deserialize
        if fmt == 1: # JSON
            return json.loads(val_bytes.decode('utf-8'))
        else:
            return val_bytes # Raw bytes

    def delete(self, key):
        """Menghapus key."""
        key_bytes = key.encode('utf-8')
        payload = struct.pack('<H', len(key_bytes)) + key_bytes
        self._send_command(0x03, payload)
        self._expect_ok()

    def subscribe(self, topic, callback):
        """
        Subscribe ke topik tertentu.
        Fungsi ini BLOCKING (loop forever), gunakan thread terpisah jika perlu.
        Callback menerima (event_type, key).
        """
        topic_bytes = topic.encode('utf-8')
        payload = struct.pack('<H', len(topic_bytes)) + topic_bytes
        self._send_command(0x20, payload) # SUBSCRIBE
        self._expect_ok()
        self.is_subscribed = True
        
        # Loop listen events
        while True:
            status, body = self._read_frame()
            if status == 0x80: # PUSH_EVENT
                # Payload: [type][topic_len][topic][key_len][key][ts]
                evt_type = body[0]
                off = 1
                tlen = struct.unpack_from('<H', body, off)[0]; off += 2
                # topic = body[off:off+tlen]; 
                off += tlen
                klen = struct.unpack_from('<H', body, off)[0]; off += 2
                key = body[off:off+klen].decode('utf-8')
                
                callback(evt_type, key)
            else:
                break

    def _send_command(self, opcode, payload):
        length = 1 + len(payload)
        req = struct.pack('<I B', length, opcode) + payload
        self.sock.sendall(req)

    def _expect_ok(self):
        status, _ = self._read_frame()
        if status != 0 and status != 4: # 4=NotFound is OK specifically? No, usually 0.
            raise Exception(f"Server Error: {status}")

# ---- CONTOH PENGGUNAAN ----
if __name__ == "__main__":
    client = LocalCachedClient()
    client.connect()

    # 1. Auto Serialize (Dictionary -> JSON)
    my_data = {"id": 101, "role": "admin", "tags": ["a", "b"]}
    client.set("py:user:101", my_data, 60000)
    print("Saved user data.")

    # 2. Auto Deserialize (JSON -> Dictionary)
    user = client.get("py:user:101")
    if user:
        print(f"User Role: {user['role']}") # Bisa langsung akses dict!

    # 3. Delete
    # 3. Delete
    client.delete("py:user:101")

    # 4. Subscribe (Blocking)
    # Gunakan thread terpisah atau process lain
    def on_event(evt_type, key):
        print(f"Update received for key: {key}")

    print("Listening for updates...")
    # client.subscribe("py:user:", on_event) 
    # ^ Uncomment baris di atas untuk start listening (Blocking)
```

---

### 2. 🟢 Node.js (Smart Wrapper)

Simpan sebagai `LocalCachedClient.js`.

```javascript
const net = require('net');

class LocalCachedClient {
    constructor(socketPath = '/tmp/localcached.sock') {
        this.socketPath = socketPath;
        this.client = null;
    }

    connect() {
        return new Promise((resolve, reject) => {
            this.client = net.createConnection(this.socketPath);
            this.client.on('connect', resolve);
            this.client.on('error', reject);
        });
    }

    /**
     * Set data. Otomatis serialize Object ke JSON.
     */
    async set(key, value, ttlMs = 0) {
        let valBuf, fmt;
        
        // Auto-Serialize
        if (typeof value === 'object') {
            valBuf = Buffer.from(JSON.stringify(value));
            fmt = 1; // JSON
        } else {
            valBuf = Buffer.from(String(value));
            fmt = 0; // Raw
        }

        const keyBuf = Buffer.from(key);
        
        // Build Payload
        const header = Buffer.alloc(2 + 2);
        header.writeUInt8(fmt, 0); 
        header.writeUInt8(0, 1);
        header.writeUInt16LE(keyBuf.length, 2);
        
        const vlen = Buffer.alloc(4);
        vlen.writeUInt32LE(valBuf.length, 0);
        
        const ttl = Buffer.alloc(8);
        ttl.writeBigUInt64LE(BigInt(ttlMs), 0);
        
        const payload = Buffer.concat([header, keyBuf, vlen, valBuf, ttl]);
        return this._send(0x01, payload);
    }

    /**
     * Get data. Otomatis deserialize JSON ke Object.
     */
    async get(key) {
        const keyBuf = Buffer.from(key);
        const payload = Buffer.concat([
            Buffer.alloc(2, keyBuf.length & 0xFFFF), // writeLE manual hack or use Buffer
            keyBuf
        ]);
        // Fix writeLE for length:
        const klen = Buffer.alloc(2); klen.writeUInt16LE(keyBuf.length);
        
        const resp = await this._send(0x02, Buffer.concat([klen, keyBuf]));
        if (!resp) return null; // NotFound

        // Parse: [fmt][vlen][val][ttl]
        const fmt = resp.readUInt8(0);
        const vlen = resp.readUInt32LE(1);
        const val = resp.slice(5, 5 + vlen);

        // Auto-Deserialize
        if (fmt === 1) return JSON.parse(val.toString());
        return val.toString();
    }

    async delete(key) {
        const keyBuf = Buffer.from(key);
        const klen = Buffer.alloc(2); klen.writeUInt16LE(keyBuf.length);
        return this._send(0x03, Buffer.concat([klen, keyBuf]));
    }

    /**
     * Subscribe to topic.
     * WARNING: This connection will be blocked for listening.
     * Use a separate connection used only for subscriptions.
     */
    async subscribe(topic, callback) {
        const tBuf = Buffer.from(topic);
        const tlen = Buffer.alloc(2); tlen.writeUInt16LE(tBuf.length);
        
        // Send SUBSCRIBE (0x20)
        await this._send(0x20, Buffer.concat([tlen, tBuf]));

        // Loop reading events
        // Note: _send() promise resolves on first packet, but we need loop here.
        // We need to bypass the standard _send request/response model for the loop.
        // For simplicity in this helper, we assume pure event loop after subscribe.
        
        this.client.on('data', (data) => {
             // Parse Event: [status=0x80][type][tlen][topic][klen][key]
             const status = data.readUInt8(4);
             if (status === 0x80) {
                 const type = data.readUInt8(5);
                 let offset = 6;
                 
                 const tLen = data.readUInt16LE(offset); offset += 2;
                 const topicStr = data.slice(offset, offset + tLen).toString(); offset += tLen;
                 
                 const kLen = data.readUInt16LE(offset); offset += 2;
                 const keyStr = data.slice(offset, offset + kLen).toString();
                 
                 callback(type, keyStr);
             }
        });
    }

    // Internal Helper
    _send(opcode, payload) {
        return new Promise((resolve, reject) => {
            const frameLen = Buffer.alloc(4);
            frameLen.writeUInt32LE(1 + payload.length, 0);
            
            const opBuf = Buffer.alloc(1);
            opBuf.writeUInt8(opcode, 0);
            
            const req = Buffer.concat([frameLen, opBuf, payload]);
            
            const onData = (data) => {
                const status = data.readUInt8(4);
                this.client.removeListener('data', onData);
                
                if (status === 4) resolve(null); // NotFound
                else if (status !== 0) reject("Error: " + status);
                else {
                    // Success, return body payload
                    resolve(data.slice(5)); 
                }
            };
            
            this.client.on('data', onData);
            this.client.write(req);
        });
    }
}

// --- CONTOH PENGGUNAAN ---
(async () => {
    const client = new LocalCachedClient();
    await client.connect();
    
    // 1. Set Object (Auto JSON)
    await client.set("node:config:1", { theme: "dark", limit: 50 });
    console.log("Config saved.");

    // 2. Get Object (Auto Parse)
    const config = await client.get("node:config:1");
    if (config) {
        console.log("User Theme:", config.theme); // Direct property access
    }

    // 3. Delete
    await client.delete("node:config:1");

    // 4. Subscribe (Blocking)
    // client.subscribe("node:config:", (type, key) => console.log("Event:", type, key));
})();
```

---

### 3. 🔷 C# .NET (Smart Helper)

Gunakan Generics `<T>` untuk auto-mapping.

```csharp
using System;
using System.Net.Sockets;
using System.Text;
using System.IO;
using System.Text.Json; // Native System.Text.Json

public class LocalCachedClient : IDisposable
{
    private Socket _socket;
    private string _path;

    public LocalCachedClient(string path = "/tmp/localcached.sock") { _path = path; }

    public void Connect() {
        var ep = new UnixDomainSocketEndPoint(_path);
        _socket = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
        _socket.Connect(ep);
    }

    // Generic SET with Auto-Serialize
    public void Set<T>(string key, T value, ulong ttlMs = 0) {
        byte format = 0;
        byte[] valBytes;

        if (typeof(T) == typeof(string)) {
            valBytes = Encoding.UTF8.GetBytes(value as string);
            format = 0;
        } else {
            // Auto JSON Serialize
            var json = JsonSerializer.Serialize(value);
            valBytes = Encoding.UTF8.GetBytes(json);
            format = 1;
        }

        SendSet(key, valBytes, ttlMs, format);
    }

    // Generic GET with Auto-Deserialize
    public T Get<T>(string key) {
        var resp = SendCommand(0x02, EncodeKeyPayload(key));
        if (resp == null) return default(T); // NotFound

        // Resp: [fmt][vlen][val]...
        using var ms = new MemoryStream(resp);
        using var br = new BinaryReader(ms);
        byte fmt = br.ReadByte();
        uint vlen = br.ReadUInt32();
        byte[] valBytes = br.ReadBytes((int)vlen);

        if (fmt == 1) {
            // Auto JSON Deserialize
            return JsonSerializer.Deserialize<T>(valBytes);
        } else {
            // Raw String
            var str = Encoding.UTF8.GetString(valBytes);
            return (T)(object)str;
        }
    }

    public void Delete(string key) {
        SendCommand(0x03, EncodeKeyPayload(key));
    }

    public void Subscribe(string topic, Action<byte, string> callback) {
        var t = Encoding.UTF8.GetBytes(topic);
        var ms = new MemoryStream();
        var bw = new BinaryWriter(ms);
        bw.Write((ushort)t.Length); bw.Write(t);
        
        // 1. Send SUBSCRIBE
        var res = SendCommand(0x20, ms.ToArray());
        // OK response handled by SendCommand
        
        // 2. Loop
        while(true) {
            // Manual Read Frame for PUSH_EVENT
            // Header [u32 len]
            var lenBuf = new byte[4]; 
            int r = _socket.Receive(lenBuf);
            if (r == 0) break; // Closed
            
            uint len = BitConverter.ToUInt32(lenBuf, 0);
            var body = new byte[len];
            int received = 0;
            while(received < len) {
                received += _socket.Receive(body, received, (int)len - received, SocketFlags.None);
            }
            
            if (body[0] == 0x80) { // PUSH_EVENT
                 // [status][type][tlen][topic][klen][key]
                 byte type = body[1];
                 int offset = 2;
                 
                 ushort tlen = BitConverter.ToUInt16(body, offset); offset += 2;
                 offset += tlen; // Skip Topic
                 
                 ushort klen = BitConverter.ToUInt16(body, offset); offset += 2;
                 var keyStr = Encoding.UTF8.GetString(body, offset, klen);
                 
                 callback(type, keyStr);
            }
        }
    }

    // --- Low Level Privates ---
    private byte[] EncodeKeyPayload(string key) {
        var k = Encoding.UTF8.GetBytes(key);
        var ms = new MemoryStream();
        var bw = new BinaryWriter(ms);
        bw.Write((ushort)k.Length);
        bw.Write(k);
        return ms.ToArray();
    }

    private void SendSet(string key, byte[] val, ulong ttl, byte fmt) {
        var k = Encoding.UTF8.GetBytes(key);
        var ms = new MemoryStream();
        var bw = new BinaryWriter(ms);
        bw.Write(fmt); bw.Write((byte)0);
        bw.Write((ushort)k.Length); bw.Write(k);
        bw.Write((uint)val.Length); bw.Write(val);
        bw.Write(ttl);
        
        var resp = SendCommand(0x01, ms.ToArray());
        if (resp == null) throw new Exception("Set Failed");
    }

    private byte[] SendCommand(byte opcode, byte[] payload) {
        uint len = (uint)(1 + payload.Length);
        _socket.Send(BitConverter.GetBytes(len)); // Assumes Little Endian arch
        _socket.Send(new byte[] { opcode });
        _socket.Send(payload);

        // Read Response
        var lenBuf = new byte[4]; _socket.Receive(lenBuf);
        uint rlen = BitConverter.ToUInt32(lenBuf, 0);
        var body = new byte[rlen];
        int r = 0; while(r<rlen) r += _socket.Receive(body, r, (int)rlen-r, SocketFlags.None);

        if (body[0] == 4) return null; // NotFound
        if (body[0] != 0) throw new Exception("Error: " + body[0]);
        
        var res = new byte[rlen - 1];
        Array.Copy(body, 1, res, 0, res.Length);
        return res;
    }

    public void Dispose() => _socket?.Dispose();
}
```

**Cara Panggil (Auto-Mapper):**

```csharp
// Domain Class
public class UserProfile {
    public int Id { get; set; }
    public string Name { get; set; }
}

// ... Main ...
using var client = new LocalCachedClient();
client.Connect();

// 1. SET Object (Auto)
var user = new UserProfile { Id = 5, Name = "Sari" };
client.Set("cs:user:5", user, 60000);

// 2. GET Object (Auto Map to Class)
UserProfile matched = client.Get<UserProfile>("cs:user:5");
Console.WriteLine($"User Name: {matched.Name}");

// 3. Delete
client.Delete("cs:user:5");

// 4. Subscribe
// client.Subscribe("cs:user:", (type, key) => Console.WriteLine($"Event: {key}"));
```

---

### 5. ☕ Java (Smart Wrapper)

Menggunakan GSON/Jackson untuk JSON (Contoh manual string manipulation untuk simplisitas).

```java
import java.net.*;
import java.nio.*;
import java.nio.channels.*;
import java.nio.charset.StandardCharsets;
import java.util.function.BiConsumer;

public class LocalCachedClient {
    private SocketChannel channel;
    private String path;

    public LocalCachedClient(String path) { this.path = path; }

    public void connect() throws Exception {
        var address = UnixDomainSocketAddress.of(path);
        channel = SocketChannel.open(address);
    }

    // Auto-Serialize (String overload for JSON)
    public void set(String key, String jsonValue, long ttlMs) throws Exception {
        byte[] k = key.getBytes(StandardCharsets.UTF_8);
        byte[] v = jsonValue.getBytes(StandardCharsets.UTF_8);
        sendSet(k, v, ttlMs, (byte)1); // fmt=1 JSON
    }

    // Raw Bytes Override
    public void set(String key, byte[] value, long ttlMs) throws Exception {
        byte[] k = key.getBytes(StandardCharsets.UTF_8);
        sendSet(k, value, ttlMs, (byte)0); // fmt=0 Raw
    }

    public String get(String key) throws Exception {
        byte[] k = key.getBytes(StandardCharsets.UTF_8);
        ByteBuffer payload = ByteBuffer.allocate(2 + k.length);
        payload.order(ByteOrder.LITTLE_ENDIAN);
        payload.putShort((short)k.length);
        payload.put(k);
        payload.flip();

        sendFrame((byte)0x02, payload);
        
        // Read Response
        byte[] body = readFrame();
        if (body == null) return null; // NotFound

        // Parse: [fmt][vlen][val]...
        byte fmt = body[0];
        int vlen = ByteBuffer.wrap(body, 1, 4).order(ByteOrder.LITTLE_ENDIAN).getInt();
        byte[] val = new byte[vlen];
        System.arraycopy(body, 5, val, 0, vlen);

        return new String(val, StandardCharsets.UTF_8); // Return JSON String
    }

    public void delete(String key) throws Exception {
        byte[] k = key.getBytes(StandardCharsets.UTF_8);
        ByteBuffer payload = ByteBuffer.allocate(2 + k.length);
        payload.order(ByteOrder.LITTLE_ENDIAN);
        payload.putShort((short)k.length).put(k).flip();
        sendFrame((byte)0x03, payload);
        readOk();
    }

    public void subscribe(String topic, BiConsumer<Byte, String> callback) throws Exception {
        byte[] t = topic.getBytes(StandardCharsets.UTF_8);
        ByteBuffer payload = ByteBuffer.allocate(2 + t.length);
        payload.order(ByteOrder.LITTLE_ENDIAN);
        payload.putShort((short)t.length).put(t).flip();
        
        sendFrame((byte)0x20, payload);
        readOk();

        // Loop Event
        while (true) {
            byte[] body = readFrame(true); // Special read allowing PUSH events
            if (body == null) break;
            
            // Parse Event: [type][tlen][topic][klen][key]...
            byte type = body[0];
            ByteBuffer bb = ByteBuffer.wrap(body).order(ByteOrder.LITTLE_ENDIAN);
            bb.position(1);
            
            short tlen = bb.getShort();
            bb.position(bb.position() + tlen); // Skip topic ref
            
            short klen = bb.getShort();
            byte[] kBytes = new byte[klen];
            bb.get(kBytes);
            
            callback.accept(type, new String(kBytes, StandardCharsets.UTF_8));
        }
    }

    // --- Private ---
    private void sendSet(byte[] k, byte[] v, long ttl, byte fmt) throws Exception {
        ByteBuffer buf = ByteBuffer.allocate(1024 + v.length);
        buf.order(ByteOrder.LITTLE_ENDIAN);
        buf.put(fmt).put((byte)0);
        buf.putShort((short)k.length).put(k);
        buf.putInt(v.length).put(v);
        buf.putLong(ttl);
        buf.flip();
        sendFrame((byte)0x01, buf);
        readOk();
    }

    private void sendFrame(byte opcode, ByteBuffer payload) throws Exception {
        int len = 1 + payload.remaining();
        ByteBuffer head = ByteBuffer.allocate(5);
        head.order(ByteOrder.LITTLE_ENDIAN);
        head.putInt(len);
        head.put(opcode);
        head.flip();
        channel.write(head);
        channel.write(payload);
    }

    private void readOk() throws Exception {
        byte[] body = readFrame();
        if (body != null && body.length > 0 && body[0] != 0) 
            throw new RuntimeException("Error: " + body[0]);
    }

    private byte[] readFrame() throws Exception { return readFrame(false); }
    private byte[] readFrame(boolean allowPush) throws Exception {
        ByteBuffer head = ByteBuffer.allocate(4);
        head.order(ByteOrder.LITTLE_ENDIAN);
        channel.read(head); head.flip();
        int len = head.getInt();

        ByteBuffer body = ByteBuffer.allocate(len);
        while (body.hasRemaining()) channel.read(body);
        body.flip();

        byte status = body.get(); // Opcode/Status
        if (allowPush && status == (byte)0x80) { // PUSH
             byte[] ret = new byte[body.remaining()];
             body.get(ret);
             return ret; // Return raw body for event parser
        }

        if (status == 4) return null; // NotFound
        if (status != 0) throw new RuntimeException("Error: " + status);
        
        byte[] ret = new byte[body.remaining()];
        body.get(ret);
        return ret;
    }
}
```

---

### 6. 🐘 PHP (Smart Wrapper)

```php
class LocalCachedClient {
    private $sock;
    private $path;

    public function __construct($path = "/tmp/localcached.sock") {
        $this->path = $path;
    }

    public function connect() {
        $this->sock = socket_create(AF_UNIX, SOCK_STREAM, 0);
        socket_connect($this->sock, $this->path);
    }

    // Auto-Serialize Array/Assoc -> JSON
    public function set($key, $val, $ttlMs = 0) {
        $fmt = 0;
        if (is_array($val) || is_object($val)) {
            $val = json_encode($val);
            $fmt = 1; // JSON
        }

        $payload = pack("CCv", $fmt, 0, strlen($key)) . $key . 
                   pack("V", strlen($val)) . $val . 
                   pack("P", $ttlMs);
        
        $this->sendFrame(0x01, $payload);
        $this->expectOk();
    }

    // Auto-Deserialize JSON -> Array
    public function get($key) {
        $payload = pack("v", strlen($key)) . $key;
        $this->sendFrame(0x02, $payload);
        
        $body = $this->readFrame();
        if ($body === null) return null; // NotFound

        // Parse: [fmt][vlen][val]...
        $fmt = unpack("C", $body)[1];
        $vlen = unpack("V", substr($body, 1))[1];
        $val = substr($body, 5, $vlen);

        if ($fmt == 1) return json_decode($val, true);
        return $val;
    }

    public function delete($key) {
        $payload = pack("v", strlen($key)) . $key;
        $this->sendFrame(0x03, $payload);
        $this->expectOk();
    }

    public function subscribe($topic, $callback) {
        $t = $topic;
        $payload = pack("v", strlen($t)) . $t;
        $this->sendFrame(0x20, $payload);
        $this->expectOk();
        
        // Loop receive
        while(true) {
            $body = $this->readFrame();
            if ($body === null) break; 
            
            // Assume frame returns body starting from status?
            // Wait, my readFrame implementation strips header but returns status?
            // Let's check readFrame implementation in the file content I have.
            // readFrame:
            // $status = unpack("C", $body)[1];
            // if ($status == 4) return null;
            // if ($status != 0) throw ...
            // return substr($body, 1);
            
            // Ah, readFrame throws if status != 0. 
            // PUSH_EVENT is 0x80 (128). So readFrame will throw "Error: 128".
            // I need to modify readFrame to allow PUSH_EVENT or handle it manually here.
            // Since readFrame is private, I can't change it easily without replacing it too.
            // I will overwrite `readFrame` as well to support passing "allowed_status".
            // OR I implement raw read logic here inside subscribe.
            
            // Re-implementing raw read for safety here:
            socket_recv($this->sock, $buf, 4, MSG_WAITALL);
            $len = unpack("V", $buf)[1];
            socket_recv($this->sock, $rawBody, $len, MSG_WAITALL);
            
            $status = unpack("C", $rawBody)[1];
            if ($status == 0x80) {
                 // Parse
                 // [status][type][tlen][topic][klen][key]
                 $type = unpack("C", substr($rawBody, 1))[1];
                 $off = 2;
                 
                 $tlen = unpack("v", substr($rawBody, $off))[1]; $off += 2;
                 $off += $tlen; // Skip Topic
                 
                 $klen = unpack("v", substr($rawBody, $off))[1]; $off += 2;
                 $key = substr($rawBody, $off, $klen);
                 
                 $callback($type, $key);
            }
        }
    }

    // --- Private ---
    private function sendFrame($opcode, $payload) {
        $len = 1 + strlen($payload);
        $head = pack("VC", $len, $opcode);
        socket_write($this->sock, $head . $payload);
    }

    private function readFrame() {
        socket_recv($this->sock, $buf, 4, MSG_WAITALL);
        $len = unpack("V", $buf)[1];
        socket_recv($this->sock, $body, $len, MSG_WAITALL);
        
        $status = unpack("C", $body)[1];
        if ($status == 4) return null;
        if ($status != 0) throw new Exception("Error: $status");
        
        return substr($body, 1);
    }

    private function expectOk() {
        $this->readFrame();
    }
}

// --- CONTOH ---
$client = new LocalCachedClient();
$client->connect();
$client->set("php:user:99", ["name" => "Rani", "active" => true], 5000);
$user = $client->get("php:user:99");
echo "User Name: " . $user['name']; 

// Subscribe (Blocking)
// $client->subscribe("php:user:", function($type, $key) {
//     echo "Got Event: $key\n";
// }); 
```

---

### 7. 🐹 Go (Smart Wrapper)

Menggunakan Interface{} untuk input generic dan `json.Unmarshal`.

```go
package main

import (
    "encoding/binary"
    "encoding/json"
    "fmt"
    "net"
    "bytes"
)

type Client struct { conn net.Conn }

func Connect(path string) (*Client, error) {
    c, err := net.Dial("unix", path)
    if err != nil { return nil, err }
    return &Client{conn: c}, nil
}

// Set: Menerima interface{}, auto convert ke JSON
func (c *Client) Set(key string, val interface{}, ttlMs uint64) error {
    var valBytes []byte
    var fmtFlag uint8 = 0
    var err error

    // Type Switch
    switch v := val.(type) {
    case string:
        valBytes = []byte(v)
    case []byte:
        valBytes = v
    default:
        valBytes, err = json.Marshal(v)
        if err != nil { return err }
        fmtFlag = 1 // JSON
    }

    buf := new(bytes.Buffer)
    buf.WriteByte(fmtFlag); buf.WriteByte(0)
    binary.Write(buf, binary.LittleEndian, uint16(len(key)))
    buf.WriteString(key)
    binary.Write(buf, binary.LittleEndian, uint32(len(valBytes)))
    buf.Write(valBytes)
    binary.Write(buf, binary.LittleEndian, ttlMs)

    return c.sendFrame(0x01, buf.Bytes())
}

// Get: Mengembalikan raw bytes & format. User bisa unmarshal sendiri manual
// atau buat helper Generic `GetJSON`
func (c *Client) Get(key string) ([]byte, bool, error) {
    buf := new(bytes.Buffer)
    binary.Write(buf, binary.LittleEndian, uint16(len(key)))
    buf.WriteString(key)

    resp, err := c.sendFrameRecv(0x02, buf.Bytes())
    if err != nil { return nil, false, err } // Error system
    if resp == nil { return nil, false, nil } // NotFound

    // Parse
    // fmt := resp[0]
    vlen := binary.LittleEndian.Uint32(resp[1:5])
    val := resp[5 : 5+vlen]
    return val, true, nil
}

func (c *Client) Delete(key string) error {
    buf := new(bytes.Buffer)
    binary.Write(buf, binary.LittleEndian, uint16(len(key)))
    buf.WriteString(key)
    return c.sendFrame(0x03, buf.Bytes())
}

// Subscribe: Blocking loop.
func (c *Client) Subscribe(topic string, callback func(uint8, string)) error {
    buf := new(bytes.Buffer)
    binary.Write(buf, binary.LittleEndian, uint16(len(topic)))
    buf.WriteString(topic)
    
    // 1. Send SUBSCRIBE
    if err := c.sendFrame(0x20, buf.Bytes()); err != nil { return err }
    // Note: sendFrame logic in this snippet reads response too.
    // Assuming sendFrame reads 1 response frame.
    // We expect OK here.
    
    // 2. Loop
    for {
        // Read Frame Header
        lenBuf := make([]byte, 4)
        if _, err := c.conn.Read(lenBuf); err != nil { return err }
        length := binary.LittleEndian.Uint32(lenBuf)
        
        body := make([]byte, length)
        if _, err := c.conn.Read(body); err != nil { return err }
        
        status := body[0]
        if status == 0x80 { // PUSH_EVENT
            // [status][type][tlen][topic][klen][key]
            evtType := body[1]
            off := 2
            
            tlen := binary.LittleEndian.Uint16(body[off:]); off += 2
            // topic := string(body[off : off+int(tlen)]); 
            off += int(tlen)
            
            klen := binary.LittleEndian.Uint16(body[off:]); off += 2
            key := string(body[off : off+int(klen)])
            
            callback(evtType, key)
        } else {
            return fmt.Errorf("unexpected status in loop: %d", status)
        }
    }
}

// --- Low Level ---
func (c *Client) sendFrame(opcode byte, payload []byte) error {
    _, err := c.sendFrameRecv(opcode, payload)
    return err
}

func (c *Client) sendFrameRecv(opcode byte, payload []byte) ([]byte, error) {
    length := uint32(1 + len(payload))
    head := make([]byte, 5)
    binary.LittleEndian.PutUint32(head, length)
    head[4] = opcode
    c.conn.Write(head)
    c.conn.Write(payload)
    
    // Read Len
    lenBuf := make([]byte, 4)
    if _, err := c.conn.Read(lenBuf); err != nil { return nil, err }
    rlen := binary.LittleEndian.Uint32(lenBuf)
    
    body := make([]byte, rlen)
    if _, err := c.conn.Read(body); err != nil { return nil, err }
    
    status := body[0]
    if status == 4 { return nil, nil } // NotFound
    if status != 0 { return nil, fmt.Errorf("server error: %d", status) }
    
    return body[1:], nil
}
```

**Cara Panggil:**

```go
func main() {
    client, _ := Connect("/tmp/localcached.sock")
    
    // 1. Set (Auto JSON)
    client.Set("go:data:1", map[string]int{"a":1, "b":2}, 5000)
    fmt.Println("Saved!")

    // 2. Get (Raw Bytes)
    val, found, _ := client.Get("go:data:1")
    if found {
        fmt.Printf("Got: %s\n", string(val))
    }

    // 3. Delete
    client.Delete("go:data:1")

    // 4. Subscribe (Blocking)
    // client.Subscribe("go:data:", func(evtType uint8, key string) {
    //    fmt.Printf("Event: %s\n", key)
    // })
}
```

---

### 8. 🦀 Rust (Native Crate)

Jika Anda membangun service lain menggunakan Rust dalam workspace yang sama (atau standalone), gunakan crate `localcached-client`.

**1. Tambahkan ke `Cargo.toml`**:

```toml
[dependencies]
localcached-client = { path = "../localcached-client" }
# Atau git path jika terpisah
tokio = { version = "1", features = ["full"] }
```

**2. Contoh Implementasi (`main.rs`)**:

```rust
use localcached_client::Client;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Connect
    let mut client = Client::connect("/tmp/localcached.sock").await?;

    // 2. Set (Raw Bytes atau Serialize manual)
    // Format Key: svc:table:pk
    client.set("rust:config:1", b"{\"mode\":\"fast\"}".to_vec(), 60000).await?;
    println!("Saved!");

    // 3. Get
    if let Some(val) = client.get("rust:config:1").await? {
        println!("Got: {:?}", String::from_utf8(val));
    }

    // 4. Delete
    client.del("rust:config:1").await?;

    // 5. Subscribe
    // Note: 'subscribe' consumes client (ownership move)
    let mut sub_stream = client.subscribe("rust:config:").await?;
    
    // Loop events
    // while let Ok(event_payload) = sub_stream.next_event().await {
    //     println!("Event Payload: {:?}", event_payload);
    // }

    Ok(())
}
```

---

### 🛠️ CLI Interaktif (Socat)

Untuk debugging cepat tanpa coding, Anda bisa mengirim raw bytes menggunakan alat-alat Unix. Namun karena protokol ini biner (menggunakan ukuran `u32` Little Endian), agak sulit mengetik manual via `nc`.

Saran terbaik untuk testing manual adalah menggunakan script Python di atas atau menggunakan `cargo run` pada `localcached-client` jika Anda membuat binary test wrapper.

---

## ⚠️ Troubleshooting

1. **Permission Denied**:
    * Pastikan user yang menjalankan client memiliki akses ke file socket (misal: `/tmp/localcached.sock`).
    * Biasanya socket dibuat dengan permission user yang menjalankan server.
    * Solusi: Jalankan client & server dengan user yang sama, atau sesuaikan `chown/chmod` pada file socket.

2. **Server Logs**:
    * Set `RUST_LOG=info` atau `RUST_LOG=debug` sebelum menjalankan server untuk melihat aktivitas detail.
    * `export RUST_LOG=debug && ./localcached-server`

3. **Invalid Key Format**:
    * Jika mendapat error saat SET, pastikan key mengandung 2 delimiter titik dua (`:`), misal `a:b:c`.

---

### 4. Memisahkan Project (Standalone Mode)

Jika Anda ingin memisahkan `sfcore-cache` dari repository utama (misal untuk deployment terpisah atau diberikan ke tim lain), ikuti panduan ini.

**Skenario**:
Kita akan memindahkan project cache ini ke folder baru.

* **Analogi Tujuan**: `C:/App/SFCore-Engine` (Atau `/opt/sfcore-engine` di Linux/Mac).

#### Langkah-langkah

1. **Siapkan Folder Tujuan**:
    Buat folder `C:/App/SFCore-Engine`.

2. **Copy 3 Komponen Utama**:
    Copy folder berikut dari `root-app/sfcore-ai/crates/` ke dalam folder tujuan:
    * `localcached-proto`
    * `localcached-server`
    * `localcached-client`

    *Struktur folder tujuan akan terlihat seperti ini:*

    ```text
    C:/App/SFCore-Engine/
    ├── localcached-proto/
    ├── localcached-server/
    ├── localcached-client/
    ```

3. **Buat File `Cargo.toml` Baru (PENTING)**:
    Agar ketiga folder tersebut bisa terhubung sebagai satu workspace, Anda **WAJIB** membuat file `Cargo.toml` di dalam `C:/App/SFCore-Engine/`.

    Isi file `C:/App/SFCore-Engine/Cargo.toml`:

    ```toml
    [workspace]
    members = [
      "localcached-proto",
      "localcached-server",
      "localcached-client",
    ]
    resolver = "2"
    ```

4. **Build Ulang**:
    Sekarang project sudah berdiri sendiri (standalone).
    Buka terminal di `C:/App/SFCore-Engine/`, lalu jalankan:

    ```bash
    # Build Server
    cargo build --release -p localcached-server
    ```

    Binary baru akan muncul di `C:/App/SFCore-Engine/target/release/localcached-server`.

---

**End of Manual**
