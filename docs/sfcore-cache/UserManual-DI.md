# 🔌 Dependency Injection & Integration Guide (UserManual-DI)

Dokumen ini menjelaskan **Praktik Terbaik** (Best Practices) untuk mengintegrasikan `localcached` ke dalam berbagai Framework aplikasi populer.

**Prinsip Utama:**

- **Singleton**: Client `localcached` harus di-instansiasi sebagai **Singleton**. Jangan membuat koneksi baru di setiap request! Koneksi UDS sangat cepat, tapi _handshake_ berulang-ulang tetap memboroskan resource.
- **Connection Lifecycle**: Buka koneksi saat aplikasi start (`Startup`), dan tutup saat aplikasi mati (`Shutdown`) jika memungkinkan.

---

## 1. 🔷 C# / .NET Core (.NET 6/7/8+)

Di ekosistem .NET, kita menggunakan `IServiceCollection` untuk Dependency Injection (DI).

### A. ASP.NET Core Web API / MVC

Buka `Program.cs`. Kita akan mendaftarkan `LocalCachedClient` sebagai service **Singleton**.

#### Langkah 1: Register Service (Program.cs)

```csharp
using YourNamespace.Services; // Namespace tempat LocalCachedClient berada

var builder = WebApplication.CreateBuilder(args);

// --- REGISTER SERVICE ---
// Singleton: Dibuat 1x saat aplikasi start, dipakai bareng-bareng.
builder.Services.AddSingleton<LocalCachedClient>(sp => 
{
    // Ambil path dari Configuration (appsettings.json) atau default
    var path = builder.Configuration["LocalCached:SocketPath"] ?? "/run/localcached.sock";
    
    var client = new LocalCachedClient(path);
    client.Connect(); // Connect langsung saat startup
    return client;
});
// ------------------------

var app = builder.Build();

// Optional: Pastikan disconnect bersih saat app shutdown
var lifetime = app.Services.GetRequiredService<IHostApplicationLifetime>();
lifetime.ApplicationStopping.Register(() => {
    var client = app.Services.GetService<LocalCachedClient>();
    client?.Dispose();
});
```

#### Langkah 2: Gunakan di Controller / Service

Cukup tambahkan di Constructor (Constructor Injection).

```csharp
[ApiController]
[Route("api/[controller]")]
public class UsersController : ControllerBase
{
    private readonly LocalCachedClient _cache;

    // DI Container otomatis meng-inject instance singleton tadi
    public UsersController(LocalCachedClient cache)
    {
        _cache = cache;
    }

    [HttpGet("{id}")]
    public IActionResult GetUser(int id)
    {
        // Gunakan cache
        var user = _cache.Get<UserProfile>($"user:{id}");
        if (user != null) return Ok(user);
        
        // ... ambil dari DB ...
        return NotFound();
    }
}
```

### B. Worker Service / Console App (IHost)

Pola yang sama berlaku jika Anda menggunakan Generic Host (`Host.CreateBuilder`).

```csharp
IHost host = Host.CreateDefaultBuilder(args)
    .ConfigureServices((hostContext, services) =>
    {
        services.AddSingleton<LocalCachedClient>(sp => {
             var client = new LocalCachedClient("/run/localcached.sock");
             client.Connect();
             return client;
        });
        
        services.AddHostedService<MyBackgroundWorker>();
    })
    .Build();

await host.RunAsync();
```

### C. Simple Console / Legacy Desktop (Static Singleton)

Jika aplikasi Anda sederhana (tanpa DI Container), gunakan pola `static readonly`.

```csharp
public static class AppCache
{
    // Lazy initialization thread-safe
    private static readonly Lazy<LocalCachedClient> _client = new Lazy<LocalCachedClient>(() => 
    {
        var c = new LocalCachedClient("/run/localcached.sock");
        c.Connect();
        return c;
    });

    public static LocalCachedClient Instance => _client.Value;
}

// Usage:
// AppCache.Instance.Set("key", "val");
```

---

## 2. 🟢 Node.js Frameworks

Node.js bersifat single-threaded event loop, jadi pattern singleton sebenarnya cukup mudah (module caching). Tapi untuk framework terstruktur, ikuti cara ini:

### A. NestJS (Modular Setup)

NestJS sangat bergantung pada Module & Provider.

#### Langkah 1: Buat Cache Module

`src/cache/cache.module.ts`

```typescript
import { Module, Global } from '@nestjs/common';
import { LocalCachedClient } from './LocalCachedClient'; // path wrapper anda

@Global() // Opsional: Biar bisa dipakai di module lain tanpa import ulang
@Module({
  providers: [
    {
      provide: 'LOCAL_CACHE', // Token Injection
      useFactory: async () => {
        const client = new LocalCachedClient('/run/localcached.sock');
        await client.connect();
        return client;
      },
    },
  ],
  exports: ['LOCAL_CACHE'],
})
export class CacheModule {}
```

#### Langkah 2: Inject di Service

`src/users/users.service.ts`

```typescript
import { Injectable, Inject } from '@nestjs/common';
// import LocalCachedClient type interface if needed

@Injectable()
export class UsersService {
  constructor(@Inject('LOCAL_CACHE') private readonly cache: any) {}

  async findOne(id: string) {
    const cached = await this.cache.get(`user:${id}`);
    if (cached) return cached;
    // ... db ...
  }
}
```

### B. Express.js / Fastify

Untuk framework minimalis, gunakan pola **Singleton Module**.

#### Langkah 1: Buat file `db.js` atau `cache.js`

```javascript
// cache.js
const { LocalCachedClient } = require('./LocalCachedClientWrapper');

const client = new LocalCachedClient('/run/localcached.sock');

// Promise koneksi -> Pastikan ditunggu sebelum server listen
const connect = async () => {
    await client.connect();
    console.log("Cache Connected");
};

module.exports = { client, connect };
```

#### Langkah 2: Init di `app.js`

```javascript
const express = require('express');
const { client, connect } = require('./cache');

const app = express();

app.get('/user/:id', async (req, res) => {
    const data = await client.get(`user:${req.params.id}`);
    res.json(data || { msg: 'Not Found' });
});

// Start server setelah connect cache
connect().then(() => {
    app.listen(3000);
});
```

---

## 3. 🐘 PHP Frameworks

PHP request lifecycle biasanya "Born & Die" per request (terutama di FPM). Namun `localcached` tetap butuh koneksi. Di PHP, kita biasanya register di Service Container.

### A. Laravel (Service Provider)

#### Langkah 1: Buat Provider

`php artisan make:provider LocalCacheServiceProvider`

#### Langkah 2: Register Singleton

Buka `app/Providers/LocalCacheServiceProvider.php`.

```php
public function register()
{
    $this->app->singleton(LocalCachedClient::class, function ($app) {
        $socketPath = env('LOCALCACHED_SOCKET', '/run/localcached.sock');
        $client = new \App\Services\LocalCachedClient($socketPath);
        $client->connect(); // Penting: Connect di sini
        return $client;
    });
}
```

#### Langkah 3: Gunakan di Controller

Laravel otomatis meng-inject via konstruktor.

```php
use App\Services\LocalCachedClient;

class UserController extends Controller
{
    protected $cache;

    public function __construct(LocalCachedClient $cache)
    {
        $this->cache = $cache;
    }

    public function show($id)
    {
        $val = $this->cache->get("user:$id");
        return response()->json($val);
    }
}
```

### B. CodeIgniter 4 (Services)

Buka/Buat `app/Config/Services.php`.

```php
namespace Config;
use CodeIgniter\Config\BaseService;
use App\Libraries\LocalCachedClient;

class Services extends BaseService
{
    public static function localcache($getShared = true)
    {
        if ($getShared) {
            return static::getSharedInstance('localcache');
        }

        $client = new LocalCachedClient('/run/localcached.sock');
        $client->connect();
        return $client;
    }
}
```

**Usage:**

```php
$cache = \Config\Services::localcache();
$data = $cache->get('key');
```

---

## 4. 🐍 Python Frameworks

### A. Django

Django tidak punya DI Container bawaan yang "strict", tapi kita bisa init di `AppConfig`.

#### Langkah 1: Init di `apps.py`

Misal di app utama `core`.

```python
# core/apps.py
from django.apps import AppConfig
from .localcached import LocalCachedClient

# Global var
cache_client = None

class CoreConfig(AppConfig):
    name = 'core'

    def ready(self):
        global cache_client
        # Mencegah double reload di dev mode
        import os
        if os.environ.get('RUN_MAIN', None) != 'true':
            return # Skip watcher process
            
        print("Connecting to Cache...")
        cache_client = LocalCachedClient("/run/localcached.sock")
        cache_client.connect()
```

#### Langkah 2: Gunakan di View

```python
# core/views.py
from .apps import cache_client

def get_user(request, id):
    if cache_client:
        user = cache_client.get(f"user:{id}")
    # ...
```

### B. FastAPI (Modern DI)

FastAPI punya sistem `Depends` yang powerful.

#### Langkah 1: Buat Dependency

```python
# dependencies.py
from .localcached import LocalCachedClient

_client = None

async def get_cache_client():
    global _client
    if _client is None:
        _client = LocalCachedClient()
        _client.connect()
    return _client
```

#### Langkah 2: Inject di Route

```python
from fastapi import FastAPI, Depends
from .dependencies import get_cache_client

app = FastAPI()

@app.get("/items/{id}")
async def read_item(id: str, cache = Depends(get_cache_client)):
    val = cache.get(f"item:{id}")
    return {"id": id, "cached": val}
```

---

## 5. 🐹 Go (Golang)

Di Go, best practice-nya adalah tidak menggunakan global variable, melainkan passing dependency via Struct (Receiver).

### A. Gin / Echo / Standard Lib

#### Langkah 1: Definisikan Struct Handler

```go
type Server struct {
    Cache *LocalCachedClient
    DB    *sql.DB
}

func (s *Server) GetUser(c *gin.Context) {
    id := c.Param("id")
    val, _ := s.Cache.Get("user:" + id)
    c.JSON(200, val)
}
```

#### Langkah 2: Wiring di `main.go`

```go
func main() {
    // 1. Init Cache
    cache := NewLocalCachedClient("/run/localcached.sock")
    err := cache.Connect()
    if err != nil { panic(err) }
    defer cache.Close()

    // 2. Inject ke Server struct
    srv := &Server{
        Cache: cache,
    }

    // 3. Setup Router
    r := gin.Default()
    r.GET("/user/:id", srv.GetUser)
    
    r.Run()
}
```

---

## 6. ☕ Java

### A. Spring Boot Integration

Spring Boot menggunakan `@Bean` untuk Dependency Injection.

#### Langkah 1: Configuration Class

```java
@Configuration
public class CacheConfig {

    @Bean
    public LocalCachedClient localCachedClient() {
        try {
            LocalCachedClient client = new LocalCachedClient("/run/localcached.sock");
            client.connect(); // Connect startup
            return client;
        } catch (Exception e) {
            throw new RuntimeException("Failed connect cache", e);
        }
    }
}
```

#### Langkah 2: Autowire di Service

```java
@Service
public class UserService {

    private final LocalCachedClient cache;

    @Autowired // Constructor Injection
    public UserService(LocalCachedClient cache) {
        this.cache = cache;
    }

    public User getUser(String id) {
        // ... use cache.get(...)
    }
}
```

---

## Kesimpulan

Apapun bahasanya, kuncinya adalah: **Buat instance client sekali saja (Singleton), lakukan koneksi di awal, dan inject instance tersebut ke bagian kode yang membutuhkan.** Hindari `new LocalCachedClient()` di dalam fungsi/method yang dipanggil berulang kali.
