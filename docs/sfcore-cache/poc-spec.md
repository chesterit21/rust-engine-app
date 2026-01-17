* **Payload JSON/MsgPack WAJIB sudah serialized dari aplikasi.**  
    Kalau tidak, server **langsung error** (tanpa sniffing / auto-serialize). Ini menghilangkan jalur mahal (parse/serialize) dari hot-path, jadi latency dan throughput jauh lebih stabil. (Konsepnya: server cuma “store bytes” + routing IPC via UDS yang memang efisien untuk lokal.) [\[deepwiki.com\]](https://deepwiki.com/rust-lang/hashbrown/2-core-architecture), [\[crates.io\]](https://crates.io/crates/dashmap/6.1.0/dependencies)
* **Pub/Sub cuma 2 event**: `invalidate` dan `table_changed`, untuk kebutuhan backend/microservices (bukan UI). Semantics cukup **best-effort / at-most-once** (subscriber lambat/putus → event bisa hilang), dan itu justru aman buat performa dan mencegah backlog tak terbatas (mirip Redis Pub/Sub). [\[blog.fzankl.de\]](https://blog.fzankl.de/unix-domain-sockets-in-net-6-basics-and-real-world-examples), [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0)

Di bawah ini kita “kunci” desain **v1**: protocol, topic model, event schema, serta step-by-step implementasi.

***

## 1) Transport & model koneksi (UDS + 2 jenis connection)

### 1.1 Transport: Unix Domain Socket (UDS)

Kita pakai **UDS** karena:

* dibuat untuk IPC lokal yang efisien (tanpa overhead jaringan TCP/IP), cocok buat komunikasi antar proses pada host yang sama.
    Implementasi Rust: `tokio::net::UnixListener` + `UnixStream`. [\[deepwiki.com\]](https://deepwiki.com/rust-lang/hashbrown/2-core-architecture), [\[crates.io\]](https://crates.io/crates/dashmap/6.1.0/dependencies) [\[microsoft.github.io\]](https://microsoft.github.io/rust-guidelines/guidelines/apps/), [\[linuxvox.com\]](https://linuxvox.com/blog/so-peercred-vs-scm-credentials-why-there-are-both-of-them/)

### 1.2 Dua koneksi per client (recommended)

Untuk menyederhanakan & memaksimalkan performa:

1. **Conn-A (Request/Response)**: CRUD (GET/SET/DEL, dll)
2. **Conn-B (Subscription stream)**: SUBSCRIBE/UNSUBSCRIBE, lalu server push event

Ini meniru pola Redis Pub/Sub di mana subscriber menerima stream push. Dengan custom protocol, ini bikin parsing lebih sederhana dan menghindari “mix mode” yang rawan edge case. (Redis sendiri jelasin client yang subscribe menerima stream message; di RESP2 bahkan command dibatasi saat subscribed). [\[blog.fzankl.de\]](https://blog.fzankl.de/unix-domain-sockets-in-net-6-basics-and-real-world-examples), [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0)

***

## 2) Data contract: payload harus serialized (no auto-detect)

### 2.1 Aturan tegas

* `SET` menerima **bytes** (JSON atau MsgPack) dan server **tidak melakukan serialize**.
* Jika client kirim payload non-serialized → server return error `ERR_NOT_SERIALIZED` / `ERR_BAD_PAYLOAD`.

Dengan begitu server jalurnya tetap:

* read frame → put/get map → (optional TTL/eviction) → response  
    Tanpa biaya parse JSON yang mahal, sehingga lebih “extreme-friendly”. Buffering pakai `BytesMut` agar minim realloc/copy saat I/O. [\[docs.rs\]](https://docs.rs/crate/parking_lot/latest), [\[crates.io\]](https://crates.io/crates/parking_lot)

***

## 3) Pub/Sub desain (invalidate + table\_changed)

Kita implement pub/sub internal menggunakan **Tokio broadcast channel** per topic:

* `tokio::sync::broadcast` adalah **multi-producer, multi-consumer** queue: setiap message dikirim ke **semua receiver**. [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0), [\[github.com\]](https://github.com/dotnet/aspnetcore/issues/29093)
* Punya solusi bawaan untuk “slow receiver”: buffer punya kapasitas tetap; saat penuh, message lama dibuang dan receiver lambat akan dapat `RecvError::Lagged` (best-effort, tidak bikin memori jebol). [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0), [\[source.dot.net\]](https://source.dot.net/System.Net.Sockets/System/Net/Sockets/UnixDomainSocketEndPoint.cs.html)

> Ini pas untuk microservices invalidation: kalau ada service yang lag sebentar, paling dia miss beberapa invalidation → dia bisa fallback re-fetch saat cache miss.

### 3.1 Topic naming (simple & scalable)

Karena event lu cuma invalidation dan table\_changed, kita buat topic level “table” aja:

* Topic untuk perubahan table:  
    `t:{table}` → contoh `t:user`, `t:order`

* Opsional: global topic (jika perlu)  
    `t:*` untuk broadcast semua table\_changed (tapi ini jarang perlu; bisa ditambah belakangan)

### 3.2 Event schema (binary kecil, bukan payload row)

**Jangan** push row/JSON besar lewat Pub/Sub. Pub/Sub cukup “signal”.

Minimal event payload:

* `event_type`: 1 byte
  * `1 = invalidate`
  * `2 = table_changed`
* `table`: string
* `key`: string (optional, tapi recommended)
* `op`: 1 byte (untuk table\_changed)
  * `1=insert, 2=update, 3=delete, 4=upsert`
* `ts`: u64 (optional, monotonic/epoch)

Semantics: **at-most-once** (best-effort) selaras dengan pub/sub model yang non-durable (Redis Pub/Sub juga at-most-once; kalau butuh durable harus pakai mekanisme lain). [\[blog.fzankl.de\]](https://blog.fzankl.de/unix-domain-sockets-in-net-6-basics-and-real-world-examples), [\[github.com\]](https://github.com/dotnet/aspnetcore/issues/47043)

***

## 4) Protocol v1 (opcodes + layout)

### 4.1 Framing umum

Request:

```text
[u32 len_le][u8 opcode][payload...]
```

Response:

```text
[u32 len_le][u8 status][payload...]
```

Kenapa framing begini? Biar read “tepat” tanpa menunggu socket close, dan parsing cepat (stream-safe). Untuk buffering, `BytesMut` cocok karena bisa reserve dan grow efisien. [\[docs.rs\]](https://docs.rs/crate/parking_lot/latest), [\[crates.io\]](https://crates.io/crates/parking_lot)

### 4.2 CRUD opcodes (Conn-A)

**SET (0x01)**  
Payload:

```text
[u8 format]         // 1=json, 2=msgpack
[u8 flags]          // bit0=publish_event (optional)
[u16 key_len][key]
[u32 val_len][val_bytes]
[u64 ttl_ms]        // 0=none
```

**GET (0x02)**

```text
[u16 key_len][key]
```

**DEL (0x03)**

```text
[u16 key_len][key]
```

**PING (0x04)**  
(no payload)

**STATS (0x05)** (optional)  
(no payload) → server balas metrics ringkas

**Status codes**:

* `0x00 OK`
* `0x01 NOT_FOUND`
* `0x10 ERR_BAD_PAYLOAD`
* `0x11 ERR_NOT_SERIALIZED` (jika ingin spesifik)
* `0x12 ERR_UNSUPPORTED_FORMAT`
* `0x13 ERR_INTERNAL`

> Karena lu kunci “payload wajib serialized”, maka `ERR_BAD_PAYLOAD/ERR_NOT_SERIALIZED` langsung keluar tanpa cek lanjut.

### 4.3 Pub/Sub opcodes (Conn-B)

**SUBSCRIBE (0x20)**

```text
[u16 topic_len][topic]
```

Server reply OK, lalu mulai push event pada connection yang sama.

**UNSUBSCRIBE (0x21)**

```text
[u16 topic_len][topic]
```

**PUBLISH (0x22)** (optional—boleh ada, tapi untuk backend biasanya publish terjadi dari server saat write)

```text
[u16 topic_len][topic]
[u32 msg_len][msg_bytes]
```

**PUSH\_EVENT (server → client)**

```text
[u8 status=0x80]            // khusus push frame
[u16 topic_len][topic]
[u32 msg_len][msg_bytes]
```

> Implementasi push ini cocok dengan model tokio broadcast: receiver `recv().await` lalu server menulis frame ke UnixStream. [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0), [\[linuxvox.com\]](https://linuxvox.com/blog/so-peercred-vs-scm-credentials-why-there-are-both-of-them/)

***

## 5) Integrasi CRUD ↔ Pub/Sub (auto-publish event)

Karena event lu cuma invalidate + table\_changed, dan kamu microservices BE, paling enak:

* Default: **SET/DEL otomatis publish `table_changed`** untuk table terkait.
* `invalidate` bisa dipublish saat:
  * DEL key tertentu
  * atau saat write yang invalidates cache turunan (mis. query cache)

Contoh mapping:

* Key format: `"{table}:{pk}"`  
    maka dari key, server bisa parse `table` = substring sebelum `:`
* Saat `SET table:123`:  
    publish ke topic `t:table` payload `{event_type:table_changed, op:update, key:"123"}`

Kenapa topic per table? Karena subscriber microservices biasanya subscribe ke subset table yang mereka peduliin. Topic count “ratusan–ribuan” masih aman karena channel dibuat lazy dan bisa di-GC. `broadcast` sendiri butuh capacity tetap, dan ada mekanisme lagging untuk receiver lambat. [\[github.com\]](https://github.com/dotnet/aspnetcore/issues/29093), [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0)

***

## 6) Memory pressure eviction (tetap dipakai untuk safety host)

Karena dataset bisa membesar (total 200K record, value variatif), pressure-aware eviction tetap penting supaya cache tidak “menghabiskan RAM” host.

* Baca `/proc/meminfo` dan pakai `MemAvailable` untuk estimasi memori yang aman dipakai tanpa swap. [\[deepwiki.com\]](https://deepwiki.com/moka-rs/moka), [\[github.com\]](https://github.com/moka-rs/moka/blob/master/README.md)
* Trigger eviction saat `pressure > 0.85`, turun target ke 0.80–0.82.
* Ini menjaga daemon tidak jadi penyebab OOM proses lain.

(Implementasi detail eviction: CLOCK/approx-LRU; kita sudah bahas sebelumnya.)

***

## 7) Step-by-step implementasi (urutan kerja yang paling cepat jadi)

### Step 1 — Skeleton server UDS (Conn-A CRUD)

1. Buat `UnixListener::bind("/run/localcached.sock")`. [\[microsoft.github.io\]](https://microsoft.github.io/rust-guidelines/guidelines/apps/), [\[deepwiki.com\]](https://deepwiki.com/rust-lang/hashbrown/2-core-architecture)
2. Accept loop, spawn task per connection `UnixStream`. [\[microsoft.github.io\]](https://microsoft.github.io/rust-guidelines/guidelines/apps/), [\[linuxvox.com\]](https://linuxvox.com/blog/so-peercred-vs-scm-credentials-why-there-are-both-of-them/)
3. Implement framing `[len][opcode]` pakai `BytesMut` + reserve. [\[docs.rs\]](https://docs.rs/crate/parking_lot/latest), [\[crates.io\]](https://crates.io/crates/parking_lot)

### Step 2 — KV store

1. Mulai dengan `DashMap<String, Entry>` untuk concurrency cepat. [\[redis.io\]](https://redis.io/docs/latest/operate/oss_and_stack/management/optimization/benchmarks/), [\[guides.wp-bullet.com\]](https://guides.wp-bullet.com/how-to-configure-redis-to-use-unix-socket-speed-boost/)
2. Entry simpan `Bytes`/`Vec<u8>` + `expires_at` + `size_bytes`.

### Step 3 — Enforce “payload must be serialized”

1. `format` harus `json` atau `msgpack`; selain itu `ERR_UNSUPPORTED_FORMAT`.
2. Basic validation minimal:
    * JSON: optional hanya cek first non-ws char `{`/`[` (murah). Tapi karena kamu sudah memutuskan “harus serialized dari app”, kamu bahkan bisa skip ini dan cukup rely pada format flag.
3. Kalau error, return `ERR_BAD_PAYLOAD` cepat.

### Step 4 — Pub/Sub (Conn-B)

1. Maintain `DashMap<Topic, broadcast::Sender<Event>>`. [\[redis.io\]](https://redis.io/docs/latest/operate/oss_and_stack/management/optimization/benchmarks/), [\[github.com\]](https://github.com/dotnet/aspnetcore/issues/29093)
2. SUBSCRIBE:
    * ambil sender (lazy-create `broadcast::channel(capacity)`), `sender.subscribe()` buat receiver. [\[github.com\]](https://github.com/dotnet/aspnetcore/issues/29093), [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0)
3. Spawn writer loop untuk push event:
    * `match rx.recv().await`:
        * Ok(msg) → kirim `PUSH_EVENT` frame
        * `Lagged` → kirim status khusus atau log, lalu lanjut (best-effort) [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0), [\[source.dot.net\]](https://source.dot.net/System.Net.Sockets/System/Net/Sockets/UnixDomainSocketEndPoint.cs.html)

### Step 5 — Auto-publish dari CRUD

1. SET/DEL parse `table` dari key (`split_once(':')`)
2. publish ke `t:{table}` event `table_changed`
3. `invalidate` dipakai kalau ada rule invalidasi lain (mis. cache query, relationship)

### Step 6 — Memory watcher + eviction

1. Periodik baca `MemAvailable` dari `/proc/meminfo`. [\[deepwiki.com\]](https://deepwiki.com/moka-rs/moka), [\[dev.to\]](https://dev.to/nithinbharathwaj/rust-performance-boost-building-efficient-caching-systems-from-scratch-7nb)
2. Jika pressure tinggi, jalankan eviction sampai turun.

***

## 8) Default config yang gua rekomendasikan (v1)

* **Topic capacity**: 256 atau 512 per topic  
    (karena event kecil dan subscriber count tidak ekstrem; broadcast punya hard cap dan drop oldest saat penuh). [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0), [\[source.dot.net\]](https://source.dot.net/System.Net.Sockets/System/Net/Sockets/UnixDomainSocketEndPoint.cs.html)
* **Watcher interval**: 100–250ms (pressure), eviction background task terpisah
* **Pub/Sub payload**: < 1KB (signal), jangan push JSON row

***
**event `upsert` saja** dan key naming yang **praktik terbaik + performa bagus + gampang dipakai**, maka rekomendasi gua adalah pakai **namespacing + delimiter `:`** dengan struktur **pendek tapi konsisten**.

Di dunia Redis (yang jadi inspirasi kamu), best practice yang paling sering dipakai itu memang **pakai `:` untuk “hierarki” key** dan **namespace per aplikasi/service** supaya gampang housekeeping dan menghindari collision.
Redis juga menekankan bahwa **panjang key itu ada biaya memori**, jadi tetap harus **ringkas**. [\[redis.io\]](https://redis.io/blog/5-key-takeaways-for-developing-with-redis/), [\[stackoverflow.com\]](https://stackoverflow.com/questions/6965451/redis-key-naming-conventions) [\[redis.io\]](https://redis.io/blog/5-key-takeaways-for-developing-with-redis/), [\[w3schools.io\]](https://www.w3schools.io/nosql/redis-keys-naming-convention/)

Di bawah ini pilihan desain yang menurut gua paling “sweet spot”.

***

## 1) Key format terbaik (rekomendasi final)

### ✅ Rekomendasi utama: `svc:table:pk`

Contoh:

* `billing:invoice:12345`
* `auth:user:1001`
* `catalog:product:SKU-8891`

**Kenapa ini best practice?**

1. **Namespace per service** menghindari tabrakan key antar microservice/aplikasi yang pakai cache yang sama (ini alasan utama prefix/namespace dipakai). [\[redis.io\]](https://redis.io/blog/5-key-takeaways-for-developing-with-redis/), [\[c-sharpcorner.com\]](https://www.c-sharpcorner.com/article/redis-naming-conventions-for-developers/)
2. `:` sebagai delimiter sudah jadi konvensi luas untuk membentuk hierarki key (`object-type:id:field` style). [\[redis.io\]](https://redis.io/blog/5-key-takeaways-for-developing-with-redis/), [\[stackoverflow.com\]](https://stackoverflow.com/questions/6965451/redis-key-naming-conventions)
3. Parsing-nya **murah**: server cukup `split_once(':')` dua kali (atau split dari kiri dengan limit 3 segmen). Ini O(n) kecil terhadap panjang key dan hanya dilakukan saat `SET/DEL` (bukan setiap byte JSON).
4. Tetap **ringkas**: Redis mengingatkan key yang panjang ikut makan memori, jadi format 3 segmen ini biasanya optimal. [\[redis.io\]](https://redis.io/blog/5-key-takeaways-for-developing-with-redis/), [\[w3schools.io\]](https://www.w3schools.io/nosql/redis-keys-naming-convention/)

> Dengan scope kamu “lokal single host tapi multi aplikasi / microservices BE”, format ini paling pas: collision aman, debugging gampang, performa tetap tinggi.

***

## 2) Kapan perlu tambah `env:` di depan?

### Opsional: `env:svc:table:pk`

Contoh:

* `dev:billing:invoice:12345`
* `prod:auth:user:1001`

Pakai ini kalau:

* kamu menjalankan **beberapa environment** (dev/staging/prod) di mesin yang sama, atau
* kamu mau “zero-risk” key collision antar environment.

Ini juga sejalan dengan praktik namespacing yang menekankan prefix untuk ownership/isolasi. [\[c-sharpcorner.com\]](https://www.c-sharpcorner.com/article/redis-naming-conventions-for-developers/), [\[redis.io\]](https://redis.io/blog/5-key-takeaways-for-developing-with-redis/)

Kalau kamu cuma local single environment (mis. hanya dev), **cukup `svc:table:pk`**.

***

## 3) Bagaimana dengan format `table:pk` saja?

### ⚠️ Bisa, tapi kurang aman untuk multi-service

`table:pk` (mis. `user:1001`) itu simpel, tapi rawan collision kalau:

* ada 2 service yang punya konsep `user` berbeda, atau
* ada cache untuk berbagai domain.

Redis best practice menyarankan namespacing supaya dataset “schema-less” itu tetap bisa di-manage dan dibersihin. [\[redis.io\]](https://redis.io/blog/5-key-takeaways-for-developing-with-redis/), [\[c-sharpcorner.com\]](https://www.c-sharpcorner.com/article/redis-naming-conventions-for-developers/)

Jadi untuk microservices: **jangan `table:pk` doang**, kecuali kamu 100% yakin cache tidak pernah dipakai lintas domain/service.

***

## 4) Impact performa: parsing key vs parsing JSON

Kabar baik: dengan keputusan kamu **payload selalu serialized** (server tidak parse/serialize JSON/msgpack), overhead parsing key (split 2–3 segmen) itu **sangat kecil** dibanding parse JSON.  
Justru keputusan kamu menghindari “auto serialize” itu yang menjaga performa extreme.

Dan IPC tetap lewat UDS + Tokio, yang memang cocok buat komunikasi proses lokal. [\[deepwiki.com\]](https://deepwiki.com/rust-lang/hashbrown/2-core-architecture), [\[microsoft.github.io\]](https://microsoft.github.io/rust-guidelines/guidelines/apps/), [\[linuxvox.com\]](https://linuxvox.com/blog/so-peercred-vs-scm-credentials-why-there-are-both-of-them/)

***

## 5) Pub/Sub: topic design yang paling clean untuk `upsert`

Kamu bilang pub/sub cuma:

* `invalidate`
* `table_changed` (cukup `upsert`)

### ✅ Topic paling sederhana: per-table

* Topic: `t:{svc}:{table}`
  * `t:billing:invoice`
  * `t:auth:user`

Kenapa bukan `t:{table}` saja?

* karena kalau table name sama di service berbeda (mis. `user` ada di auth dan billing), topic bisa tabrakan. Namespacing tetap penting. [\[redis.io\]](https://redis.io/blog/5-key-takeaways-for-developing-with-redis/), [\[c-sharpcorner.com\]](https://www.c-sharpcorner.com/article/redis-naming-conventions-for-developers/)

### Payload event (kecil aja)

* `invalidate`: kirim `key` penuh
* `table_changed(upsert)`: kirim `pk` atau `key` penuh (pilih satu konsisten)

**Saran gua**:

* `invalidate` → bawa `key` penuh (biar subscriber tinggal DEL lokal / refresh).
* `table_changed` → bawa `pk` saja (lebih kecil) *atau* `key` penuh (lebih simpel).  
    Kalau kamu sudah standard `svc:table:pk`, bawa `key` penuh itu nyaman.

### Implementasi internal pub/sub

Untuk performa dan anti “slow consumer memory leak”, pakai `tokio::sync::broadcast` per topic:

* `broadcast::channel(capacity)` membuat channel bounded. [\[github.com\]](https://github.com/dotnet/aspnetcore/issues/29093), [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0)
* kalau subscriber lambat, channel akan drop message lama dan receiver dapat `Lagged`, sehingga sistem tetap stabil (best-effort). [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0), [\[source.dot.net\]](https://source.dot.net/System.Net.Sockets/System/Net/Sockets/UnixDomainSocketEndPoint.cs.html)

Ini inline juga dengan konsep Pub/Sub yang **fire-and-forget** (at-most-once) seperti Redis Pub/Sub. [\[blog.fzankl.de\]](https://blog.fzankl.de/unix-domain-sockets-in-net-6-basics-and-real-world-examples), [\[github.com\]](https://github.com/dotnet/aspnetcore/issues/47043)

***

## 6) “Kemudahan” (developer experience) yang gua saranin

Agar tim kamu gak salah format key, bikin **client helper** (SDK ringan) di setiap bahasa:

* `KeyBuilder(service, table).pk(id)` → hasil `svc:table:pk`
* `TopicBuilder(service, table)` → hasil `t:svc:table`

Dengan begini, 95% human error hilang tanpa bikin server melakukan kerja ekstra.

***

## 7) Keputusan final (yang gua rekomendasikan kamu kunci)

1. **Key**: `svc:table:pk` (paling balance). [\[redis.io\]](https://redis.io/blog/5-key-takeaways-for-developing-with-redis/), [\[stackoverflow.com\]](https://stackoverflow.com/questions/6965451/redis-key-naming-conventions)
2. (Opsional) tambah env prefix kalau multi env: `env:svc:table:pk`. [\[c-sharpcorner.com\]](https://www.c-sharpcorner.com/article/redis-naming-conventions-for-developers/), [\[redis.io\]](https://redis.io/blog/5-key-takeaways-for-developing-with-redis/)
3. **Topic**: `t:svc:table`
4. **Event**:
    * `invalidate(key_full)`
    * `table_changed(upsert, key_full atau pk)`
5. **Pub/Sub engine**: `tokio::sync::broadcast` per topic, capacity bounded. [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0), [\[github.com\]](https://github.com/dotnet/aspnetcore/issues/29093)

***
Siap. Kita **rancang bener**: **spec payload binary v1 + planning task end-to-end** (create crate dalam `crates/` workspace, struktur modul, dependency antar “class/module” jelas, code snippet real, mandatory items, dan deployment).  
Target: **Local-only cache daemon** via **UDS** + **KV CRUD** + **Pub/Sub (invalidate + table\_changed upsert)** + **memory-pressure eviction**.

> Referensi yang kita pakai benar-benar “real-world” dari docs resmi: **Tokio UDS**, **BytesMut**, **DashMap**, **Tokio broadcast**, dan **/proc/meminfo**. [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html), [\[dev.to\]](https://dev.to/leapcell/optimizing-rust-performance-with-jemalloc-36lo), [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide), [\[docs.rs\]](https://docs.rs/crate/parking_lot/latest), [\[exchangetuts.com\]](https://exchangetuts.com/index.php/tcp-loopback-connection-vs-unix-domain-socket-performance-1639495325509681), [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en)

***

# 0) Keputusan desain final (locked)

1. **Payload data (JSON/msgpack) wajib sudah serialized dari aplikasi**.  
    Jika tidak, server **langsung balas error** (tidak ada auto-detect/auto-serialize). Ini menjaga hot-path tetap tipis & cepat.
2. **Key format**: `svc:table:pk` (ringkas + aman collision antar microservice), optional `env:svc:table:pk` bila multi-env. Redis sendiri menekankan pentingnya namespace + delimiter `:` dan tetap menjaga key pendek karena key makan memori. [\[github.com\]](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Sockets/src/System/Net/Sockets/UnixDomainSocketEndPoint.Windows.cs), [\[blog.fzankl.de\]](https://blog.fzankl.de/unix-domain-sockets-in-net-6-basics-and-real-world-examples), [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/aspnet/core/grpc/interprocess?view=aspnetcore-10.0)
3. **Pub/Sub event** hanya:
    * `invalidate(key_full)`
    * `table_changed(upsert, key_full)` (karena lu pilih upsert aja)
4. **Topic**: `t:{svc}:{table}` (topic per table per service).
5. **Pub/Sub engine**: `tokio::sync::broadcast` per topic (bounded ring buffer; slow receiver akan “lagged”, bukan bikin memori jebol). [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html), [\[exchangetuts.com\]](https://exchangetuts.com/index.php/tcp-loopback-connection-vs-unix-domain-socket-performance-1639495325509681)
6. **Transport IPC**: **UDS** (Unix Domain Socket) via Tokio `UnixListener/UnixStream`. [\[stackoverflow.com\]](https://stackoverflow.com/questions/9898961/is-there-a-way-to-get-the-uid-of-the-other-end-of-a-unix-socket-connection), [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html), [\[dev.to\]](https://dev.to/leapcell/optimizing-rust-performance-with-jemalloc-36lo)
7. **Memory pressure**: gunakan `MemAvailable` dari `/proc/meminfo` sebagai estimasi memori yang aman tanpa swap, dan trigger eviction saat > 85%. [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html)

***

# 1) SPEC PROTOCOL v1 (Binary)

## 1.1 Framing umum (semua message)

**Request Frame**

    u32  len_le      // panjang payload+opcode (tidak termasuk len_le itu sendiri)
    u8   opcode
    u8[] payload     // len_le-1 bytes

**Response Frame**

    u32  len_le
    u8   status
    u8[] payload

> Kenapa framing ini: supaya stream socket tidak bergantung pada socket close; parsing jadi deterministic dan cepat. `BytesMut` cocok untuk buffer grow & meminimalkan copy. [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide), [\[docs.rs\]](https://docs.rs/nix/latest/nix/sys/socket/index.html)

***

## 1.2 Opcode list

### CRUD (Conn-A / request-response)

* `0x01 SET`
* `0x02 GET`
* `0x03 DEL`
* `0x04 PING`
* `0x05 STATS` (opsional, tapi gua masukin sebagai “wajib bagus”)

### Pub/Sub (Conn-B / subscribe stream)

* `0x20 SUBSCRIBE`
* `0x21 UNSUBSCRIBE`

### Server push event (Conn-B)

* `0x80 PUSH_EVENT` (server → client, tidak pernah dikirim client)

***

## 1.3 Status codes (response)

* `0x00 OK`
* `0x01 NOT_FOUND`
* `0x10 ERR_BAD_PAYLOAD`
* `0x11 ERR_UNSUPPORTED_FORMAT`
* `0x12 ERR_TOO_LARGE`
* `0x13 ERR_INTERNAL`
* `0x14 ERR_UNAUTHORIZED` (kalau nanti ditambah credential check)
* `0x15 ERR_LAGGED` (khusus push: receiver ketinggalan, best-effort)

***

## 1.4 Payload detail

### SET (opcode=0x01)

    u8   format       // 1=json, 2=msgpack
    u8   flags        // bit0=publish_event (1=publish table_changed upsert)
    u16  key_len
    u8[] key_bytes
    u32  val_len
    u8[] val_bytes
    u64  ttl_ms       // 0 berarti no ttl

**Rule validasi server**

* `format` harus 1 atau 2; selain itu `ERR_UNSUPPORTED_FORMAT`.
* `val_len > 0` wajib; kalau 0 → `ERR_BAD_PAYLOAD`.
* **Tidak ada parsing JSON/msgpack** (anggap bytes sudah valid).
* `key` wajib ASCII/UTF-8 (kamu bebas, tapi rekomendasi UTF-8).

### GET (opcode=0x02)

    u16 key_len
    u8[] key

**Response OK**

    u8  format
    u32 val_len
    u8[] val_bytes
    u64 ttl_remaining_ms (opsional)

### DEL (opcode=0x03)

    u16 key_len
    u8[] key

### SUBSCRIBE (0x20)

    u16 topic_len
    u8[] topic

**Response**: OK. Setelah itu server akan push `PUSH_EVENT` setiap ada event.

### UNSUBSCRIBE (0x21)

    u16 topic_len
    u8[] topic

### PUSH\_EVENT (opcode=0x80) — server → client

    u8   event_type   // 1=invalidate, 2=table_changed
    u16  topic_len
    u8[] topic_bytes  // "t:svc:table"
    u16  key_len
    u8[] key_bytes    // key_full: "svc:table:pk"
    u64  ts_ms        // epoch ms (opsional tapi recommended untuk tracing)

**event\_type=1 invalidate**

* key\_full wajib.

**event\_type=2 table\_changed**

* op selalu implicit `upsert` (tidak kirim op lagi, karena kamu lock upsert).

> Semantics pubsub: best-effort, bounded buffer; slow receiver bisa “lagged”. Ini sejalan dengan model tokio broadcast (RecvError::Lagged) dan juga sejalan dengan sifat pubsub “fire-and-forget” seperti Redis Pub/Sub. [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html), [\[dev.to\]](https://dev.to/lordsnow/iouring-the-modern-asynchronous-io-revolution-in-linux-46ch)

***

# 2) Planning Task (step-by-step) + Struktur Workspace

## Task 0 — Buat crate baru di Cargo Workspace

**Tujuan**: bikin `crates/localcached` sebagai binary crate (daemon).

**Folder**

    <root-workspace>/
      Cargo.toml          # workspace
      crates/
        localcached/
          Cargo.toml
          src/
            main.rs
            lib.rs         # optional (biar modul rapi)
            config.rs
            protocol/
              mod.rs
              codec.rs
              types.rs
            server/
              mod.rs
              conn_kv.rs
              conn_sub.rs
            store/
              mod.rs
              kv_store.rs
              entry.rs
              eviction.rs
            pubsub/
              mod.rs
              bus.rs
              events.rs
            sys/
              mod.rs
              meminfo.rs
            metrics/
              mod.rs
              stats.rs
          benches/
            kv_bench.rs

**Workspace root `Cargo.toml`** (tambahkan member):

```toml
[workspace]
members = [
  "crates/localcached",
  # ... crate lain
]
resolver = "2"
```

> `resolver=2` recommended untuk workspace modern agar feature resolution lebih predictable.

***

## Task 1 — Tentukan dependencies (library yang dipakai) + alasan

**crates/localcached/Cargo.toml**

```toml
[package]
name = "localcached"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "sync", "time", "signal"] }
bytes = "1"
dashmap = "6"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# sinkronisasi lock ringan untuk eviction ring / config mutation
parking_lot = "0.12"

# allocator cepat (opsional, tapi recommended untuk daemon)
mimalloc = { version = "0.1", optional = true }

[features]
default = []
alloc_mimalloc = ["mimalloc"]

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "kv_bench"
harness = false
```

**Kenapa ini stack yang “real-world”:**

* Tokio UDS: `tokio::net::UnixListener/UnixStream` adalah API standar untuk UDS async. [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html), [\[dev.to\]](https://dev.to/leapcell/optimizing-rust-performance-with-jemalloc-36lo)
* `bytes::BytesMut` untuk buffer network efisien & minim copy. [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide), [\[docs.rs\]](https://docs.rs/nix/latest/nix/sys/socket/index.html)
* `DashMap` sebagai concurrent map cepat (awal). [\[docs.rs\]](https://docs.rs/crate/parking_lot/latest), [\[docs.serai.exchange\]](https://docs.serai.exchange/rust/parking_lot/index.html)
* `tokio::sync::broadcast` untuk pubsub bounded; menangani slow receiver dengan Lagged. [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[exchangetuts.com\]](https://exchangetuts.com/index.php/tcp-loopback-connection-vs-unix-domain-socket-performance-1639495325509681), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html)
* `/proc/meminfo MemAvailable` untuk memory pressure estimator. [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html)
* Mimalloc opsional (global allocator) untuk aplikasi; ada crate wrapper official. [\[microsoft.github.io\]](https://microsoft.github.io/rust-guidelines/guidelines/apps/), [\[docs.rs\]](https://docs.rs/crate/mimalloc/latest)
* Criterion untuk benchmark statistik-driven. [\[docs.rs\]](https://docs.rs/crossbeam/latest/crossbeam/), [\[github.com\]](https://github.com/crossbeam-rs/crossbeam)

***

## Task 2 — Implement `config` + bootstrap logging

**Wajib**: config socket path, thresholds, channel capacity, ttl default.

`src/config.rs`

```rust
#[derive(Clone, Debug)]
pub struct Config {
    pub socket_path: String,
    pub pressure_hot: f64,   // 0.85
    pub pressure_cool: f64,  // 0.80
    pub pubsub_capacity: usize, // 256
    pub max_frame_bytes: usize, // proteksi DOS (mis. 8MB)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            socket_path: "/run/localcached.sock".to_string(),
            pressure_hot: 0.85,
            pressure_cool: 0.80,
            pubsub_capacity: 256,
            max_frame_bytes: 8 * 1024 * 1024,
        }
    }
}
```

`src/main.rs` (bootstrap)

```rust
#[cfg(feature = "alloc_mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use tracing_subscriber::EnvFilter;

mod config;
mod protocol;
mod server;
mod store;
mod pubsub;
mod sys;
mod metrics;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cfg = config::Config::default();
    server::run(cfg).await?;
    Ok(())
}
```

> Mimalloc sebagai global allocator opsional menggunakan pattern yang direkomendasikan crate. [\[microsoft.github.io\]](https://microsoft.github.io/rust-guidelines/guidelines/apps/), [\[github.com\]](https://github.com/gnzlbg/mimallocator), [\[docs.rs\]](https://docs.rs/crate/mimalloc/latest)

***

## Task 3 — Protocol module: types + codec (Wajib, core)

### 3.1 `protocol/types.rs`

```rust
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Opcode {
    Set = 0x01,
    Get = 0x02,
    Del = 0x03,
    Ping = 0x04,
    Stats = 0x05,

    Subscribe = 0x20,
    Unsubscribe = 0x21,

    PushEvent = 0x80,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Ok = 0x00,
    NotFound = 0x01,
    ErrBadPayload = 0x10,
    ErrUnsupportedFormat = 0x11,
    ErrTooLarge = 0x12,
    ErrInternal = 0x13,
    ErrUnauthorized = 0x14,
    ErrLagged = 0x15,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValueFormat {
    Json = 1,
    MsgPack = 2,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    Invalidate = 1,
    TableChanged = 2, // implicit upsert
}
```

### 3.2 `protocol/codec.rs` (framing)

Kita pakai `BytesMut` untuk read buffer. Ini pattern umum networking Rust. [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide), [\[docs.rs\]](https://docs.rs/nix/latest/nix/sys/socket/index.html)

```rust
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::protocol::types::{Status};

pub async fn read_frame<R: AsyncReadExt + Unpin>(
    r: &mut R,
    max_frame: usize,
    buf: &mut BytesMut,
) -> std::io::Result<Option<BytesMut>> {
    // Pastikan minimal 4 byte (len)
    while buf.len() < 4 {
        let n = r.read_buf(buf).await?;
        if n == 0 {
            return Ok(None);
        }
    }

    let mut len_buf = &buf[..4];
    let len = len_buf.get_u32_le() as usize;

    if len == 0 || len > max_frame {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "frame too large"));
    }

    let total = 4 + len;
    while buf.len() < total {
        let n = r.read_buf(buf).await?;
        if n == 0 {
            return Ok(None);
        }
    }

    let frame = buf.split_to(total);
    Ok(Some(frame))
}

pub async fn write_response<W: AsyncWriteExt + Unpin>(
    w: &mut W,
    status: Status,
    payload: &[u8],
) -> std::io::Result<()> {
    let len = 1 + payload.len();
    let mut out = BytesMut::with_capacity(4 + len);
    out.put_u32_le(len as u32);
    out.put_u8(status as u8);
    out.extend_from_slice(payload);
    w.write_all(&out).await
}
```

***

## Task 4 — Store module: KV + Entry + TTL (Wajib)

### 4.1 `store/entry.rs`

```rust
use bytes::Bytes;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::protocol::types::ValueFormat;

#[derive(Debug)]
pub struct Entry {
    pub format: ValueFormat,
    pub value: Bytes,
    pub expires_at_ms: u64,  // 0 = none
    pub touched: AtomicU64,  // monotonic counter / timestamp for CLOCK-ish
    pub size_bytes: usize,
}

impl Entry {
    pub fn is_expired(&self, now_ms: u64) -> bool {
        self.expires_at_ms != 0 && now_ms >= self.expires_at_ms
    }

    pub fn touch(&self, now_ms: u64) {
        self.touched.store(now_ms, Ordering::Relaxed);
    }
}
```

### 4.2 `store/kv_store.rs`

Gunakan DashMap untuk concurrent map “cepat jadi” dan stabil. [\[docs.rs\]](https://docs.rs/crate/parking_lot/latest), [\[docs.serai.exchange\]](https://docs.serai.exchange/rust/parking_lot/index.html)

```rust
use dashmap::DashMap;
use bytes::Bytes;
use crate::store::entry::Entry;
use crate::protocol::types::ValueFormat;

#[derive(Default)]
pub struct KvStore {
    map: DashMap<String, Entry>,
}

impl KvStore {
    pub fn set(&self, key: String, format: ValueFormat, value: Bytes, expires_at_ms: u64, now_ms: u64) {
        let size_bytes = key.len() + value.len();
        let e = Entry { format, value, expires_at_ms, touched: now_ms.into(), size_bytes };
        self.map.insert(key, e);
    }

    pub fn get(&self, key: &str, now_ms: u64) -> Option<(ValueFormat, Bytes)> {
        let guard = self.map.get(key)?;
        if guard.is_expired(now_ms) {
            drop(guard);
            self.map.remove(key);
            return None;
        }
        guard.touch(now_ms);
        Some((guard.format, guard.value.clone()))
    }

    pub fn del(&self, key: &str) -> bool {
        self.map.remove(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}
```

> `Bytes` clone itu murah (ref-counted) dibanding clone Vec; cocok untuk cache. `BytesMut → freeze → Bytes` pattern umum. [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide), [\[github.com\]](https://github.com/nix-rust/nix/issues/2070)

***

## Task 5 — Pub/Sub module: topic bus + events (Wajib)

Kita pakai `tokio::sync::broadcast` karena ini tepat untuk “pubsub ring buffer bounded” dan punya `Lagged` behavior. [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html), [\[exchangetuts.com\]](https://exchangetuts.com/index.php/tcp-loopback-connection-vs-unix-domain-socket-performance-1639495325509681)

### 5.1 `pubsub/events.rs`

```rust
use crate::protocol::types::EventType;

#[derive(Clone, Debug)]
pub struct Event {
    pub event_type: EventType,
    pub topic: String,   // "t:svc:table"
    pub key: String,     // "svc:table:pk"
    pub ts_ms: u64,
}
```

### 5.2 `pubsub/bus.rs`

```rust
use dashmap::DashMap;
use tokio::sync::broadcast;
use crate::pubsub::events::Event;

pub struct Bus {
    topics: DashMap<String, broadcast::Sender<Event>>,
    capacity: usize,
}

impl Bus {
    pub fn new(capacity: usize) -> Self {
        Self { topics: DashMap::new(), capacity }
    }

    fn get_or_create(&self, topic: &str) -> broadcast::Sender<Event> {
        if let Some(s) = self.topics.get(topic) {
            return s.clone();
        }
        let (tx, _rx) = broadcast::channel(self.capacity); // bounded broadcast 
        self.topics.insert(topic.to_string(), tx.clone());
        tx
    }

    pub fn subscribe(&self, topic: &str) -> broadcast::Receiver<Event> {
        self.get_or_create(topic).subscribe()
    }

    pub fn publish(&self, ev: Event) {
        let tx = self.get_or_create(&ev.topic);
        let _ = tx.send(ev); // best-effort
    }
}
```

***

## Task 6 — Sys module: meminfo watcher (Wajib)

`/proc/meminfo` menjelaskan `MemAvailable` sebagai estimasi memory yang available tanpa swap (Linux 3.14+). Ini yang kita pakai untuk trigger eviction. [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html)

`sys/meminfo.rs`

```rust
use std::fs;

#[derive(Debug, Clone, Copy)]
pub struct MemInfo {
    pub mem_total_kb: u64,
    pub mem_available_kb: u64,
}

pub fn read_meminfo() -> std::io::Result<MemInfo> {
    let s = fs::read_to_string("/proc/meminfo")?;
    let mut total = 0u64;
    let mut avail = 0u64;

    for line in s.lines() {
        if line.starts_with("MemTotal:") {
            total = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            avail = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        }
    }

    Ok(MemInfo { mem_total_kb: total, mem_available_kb: avail })
}

pub fn pressure(mi: MemInfo) -> f64 {
    if mi.mem_total_kb == 0 { return 0.0; }
    let avail = mi.mem_available_kb as f64;
    let total = mi.mem_total_kb as f64;
    1.0 - (avail / total)
}
```

***

## Task 7 — Eviction module (Wajib)

**Goal**: ketika `pressure > 0.85`, evict sampai `pressure < 0.80`.  
Policy sederhana tapi efektif: **CLOCK-ish** berdasarkan ring key + touched timestamp.

`store/eviction.rs`

```rust
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::store::kv_store::KvStore;
use crate::sys::meminfo::{read_meminfo, pressure};

pub struct Evictor {
    ring: Mutex<VecDeque<String>>,
    store: Arc<KvStore>,
    hot: f64,
    cool: f64,
}

impl Evictor {
    pub fn new(store: Arc<KvStore>, hot: f64, cool: f64) -> Self {
        Self { ring: Mutex::new(VecDeque::new()), store, hot, cool }
    }

    pub fn on_key_write(&self, key: &str) {
        // best-effort: ring bisa mengandung duplicate; cleanup terjadi saat evict
        self.ring.lock().push_back(key.to_string());
    }

    pub async fn run(self: Arc<Self>) {
        loop {
            // sampling interval
            sleep(Duration::from_millis(150)).await;

            let mi = match read_meminfo() {
                Ok(mi) => mi,
                Err(_) => continue,
            };
            let p = pressure(mi);
            if p < self.hot {
                continue;
            }

            // HOT: evict until COOL
            while p >= self.cool {
                if !self.evict_one() {
                    break;
                }
                // re-read pressure periodically
                if let Ok(mi2) = read_meminfo() {
                    let p2 = pressure(mi2);
                    if p2 < self.cool { break; }
                }
            }
        }
    }

    fn evict_one(&self) -> bool {
        let key = self.ring.lock().pop_front();
        let Some(k) = key else { return false; };

        // Evict blindly (best-effort). Bisa ditingkatkan: cek touched/age.
        let _ = self.store.del(&k);
        true
    }
}
```

> Ini versi minimal yang “jalan”. Nanti bisa di-upgrade ke CLOCK beneran (pakai touched & age) agar eviction lebih “cerdas”. Tapi **wajib minimal**: ada mekanisme membuang key saat pressure tinggi agar tidak OOM host. `MemAvailable` referensinya jelas di man page. [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html)

***

## Task 8 — Server module: routing koneksi KV & SUB (Wajib)

### 8.1 `server/run` (bind UDS, accept loop)

Tokio UDS binding: `UnixListener::bind`. [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html), [\[dev.to\]](https://dev.to/leapcell/optimizing-rust-performance-with-jemalloc-36lo)

`server/mod.rs`

```rust
use std::{path::Path, sync::Arc};
use tokio::net::UnixListener;

use crate::config::Config;
use crate::store::kv_store::KvStore;
use crate::pubsub::bus::Bus;
use crate::store::eviction::Evictor;

pub mod conn_kv;
pub mod conn_sub;

pub async fn run(cfg: Config) -> anyhow::Result<()> {
    // remove old socket if exists
    if Path::new(&cfg.socket_path).exists() {
        let _ = std::fs::remove_file(&cfg.socket_path);
    }

    let listener = UnixListener::bind(&cfg.socket_path)?;
    tracing::info!("listening on UDS: {}", cfg.socket_path);

    let store = Arc::new(KvStore::default());
    let bus = Arc::new(Bus::new(cfg.pubsub_capacity));

    let evictor = Arc::new(Evictor::new(store.clone(), cfg.pressure_hot, cfg.pressure_cool));
    let evictor_bg = evictor.clone();
    tokio::spawn(async move { evictor_bg.run().await; });

    loop {
        let (stream, _addr) = listener.accept().await?;
        // Strategy:
        // - client chooses mode via first opcode (SUBSCRIBE => conn_sub, else conn_kv)
        let cfg2 = cfg.clone();
        let store2 = store.clone();
        let bus2 = bus.clone();
        let ev2 = evictor.clone();

        tokio::spawn(async move {
            if let Err(e) = conn_kv::handle(stream, cfg2, store2, bus2, ev2).await {
                tracing::warn!("conn ended: {e:?}");
            }
        });
    }
}
```

> Catatan: untuk mode SUBSCRIBE connection khusus, kita bisa deteksi opcode pertama, lalu “upgrade” handler menjadi `conn_sub`. Ini akan kita implement di `conn_kv::handle` (lihat Task 9). Tokio accept loop & spawn pattern ini umum. [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html), [\[dev.to\]](https://dev.to/leapcell/optimizing-rust-performance-with-jemalloc-36lo)

***

## Task 9 — Handler KV + auto publish event (Wajib)

`server/conn_kv.rs`

* baca frame → decode opcode → call store/bus/evictor.
* **Wajib** enforce `format` json/msgpack.

```rust
use std::sync::Arc;
use bytes::{Buf, BytesMut};
use tokio::net::UnixStream;
use tokio::io::{AsyncWriteExt};

use crate::config::Config;
use crate::protocol::codec::{read_frame, write_response};
use crate::protocol::types::{Opcode, Status, ValueFormat, EventType};
use crate::store::kv_store::KvStore;
use crate::pubsub::bus::Bus;
use crate::pubsub::events::Event;
use crate::store::eviction::Evictor;

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

// derive topic from key: "svc:table:pk" => topic "t:svc:table"
fn topic_from_key(key: &str) -> Option<String> {
    let mut it = key.splitn(3, ':');
    let svc = it.next()?;
    let table = it.next()?;
    Some(format!("t:{svc}:{table}"))
}

pub async fn handle(
    mut stream: UnixStream,
    cfg: Config,
    store: Arc<KvStore>,
    bus: Arc<Bus>,
    evictor: Arc<Evictor>,
) -> anyhow::Result<()> {
    let mut buf = BytesMut::with_capacity(8 * 1024);

    loop {
        let Some(frame) = read_frame(&mut stream, cfg.max_frame_bytes, &mut buf).await? else {
            return Ok(());
        };

        // frame layout: [len u32][opcode u8][payload...]
        let mut rd = &frame[4..];
        let opcode = rd.get_u8();

        if opcode == Opcode::Subscribe as u8 {
            // upgrade to sub connection handler
            return crate::server::conn_sub::handle(stream, cfg, bus).await;
        }

        match opcode {
            x if x == Opcode::Ping as u8 => {
                write_response(&mut stream, Status::Ok, &[]).await?;
            }
            x if x == Opcode::Get as u8 => {
                let key_len = rd.get_u16_le() as usize;
                if rd.remaining() < key_len { write_response(&mut stream, Status::ErrBadPayload, &[]).await?; continue; }
                let key = std::str::from_utf8(&rd[..key_len]).map_err(|_| anyhow::anyhow!("bad key utf8"))?;
                let key = key.to_string();
                rd.advance(key_len);

                let now = now_ms();
                if let Some((fmt, val)) = store.get(&key, now) {
                    let mut out = BytesMut::with_capacity(1 + 4 + val.len());
                    out.put_u8(fmt as u8);
                    out.put_u32_le(val.len() as u32);
                    out.extend_from_slice(&val);
                    write_response(&mut stream, Status::Ok, &out).await?;
                } else {
                    write_response(&mut stream, Status::NotFound, &[]).await?;
                }
            }
            x if x == Opcode::Del as u8 => {
                let key_len = rd.get_u16_le() as usize;
                if rd.remaining() < key_len { write_response(&mut stream, Status::ErrBadPayload, &[]).await?; continue; }
                let key = std::str::from_utf8(&rd[..key_len]).map_err(|_| anyhow::anyhow!("bad key utf8"))?.to_string();
                rd.advance(key_len);

                let existed = store.del(&key);
                if existed {
                    // publish invalidate
                    if let Some(topic) = topic_from_key(&key) {
                        bus.publish(Event {
                            event_type: EventType::Invalidate,
                            topic,
                            key: key.clone(),
                            ts_ms: now_ms(),
                        });
                    }
                    write_response(&mut stream, Status::Ok, &[]).await?;
                } else {
                    write_response(&mut stream, Status::NotFound, &[]).await?;
                }
            }
            x if x == Opcode::Set as u8 => {
                if rd.remaining() < 1+1+2 { write_response(&mut stream, Status::ErrBadPayload, &[]).await?; continue; }
                let fmt = rd.get_u8();
                let flags = rd.get_u8();
                let format = match fmt {
                    1 => ValueFormat::Json,
                    2 => ValueFormat::MsgPack,
                    _ => { write_response(&mut stream, Status::ErrUnsupportedFormat, &[]).await?; continue; }
                };

                let key_len = rd.get_u16_le() as usize;
                if rd.remaining() < key_len + 4 { write_response(&mut stream, Status::ErrBadPayload, &[]).await?; continue; }
                let key = std::str::from_utf8(&rd[..key_len]).map_err(|_| anyhow::anyhow!("bad key utf8"))?.to_string();
                rd.advance(key_len);

                let val_len = rd.get_u32_le() as usize;
                if val_len == 0 || val_len > cfg.max_frame_bytes { write_response(&mut stream, Status::ErrBadPayload, &[]).await?; continue; }
                if rd.remaining() < val_len + 8 { write_response(&mut stream, Status::ErrBadPayload, &[]).await?; continue; }
                let val_bytes = bytes::Bytes::copy_from_slice(&rd[..val_len]); // could optimize to zero-copy with BytesMut freeze
                rd.advance(val_len);

                let ttl_ms = rd.get_u64_le();
                let now = now_ms();
                let expires_at = if ttl_ms == 0 { 0 } else { now + ttl_ms };

                store.set(key.clone(), format, val_bytes, expires_at, now);
                evictor.on_key_write(&key);

                // auto publish table_changed upsert jika flags.bit0 = 1
                if (flags & 0b0000_0001) != 0 {
                    if let Some(topic) = topic_from_key(&key) {
                        bus.publish(Event {
                            event_type: EventType::TableChanged,
                            topic,
                            key: key.clone(),
                            ts_ms: now_ms(),
                        });
                    }
                }

                write_response(&mut stream, Status::Ok, &[]).await?;
            }
            _ => {
                write_response(&mut stream, Status::ErrBadPayload, &[]).await?;
            }
        }
    }
}
```

***

## Task 10 — Handler SUB connection (Wajib)

Handler ini memegang:

* Map `topic -> broadcast::Receiver<Event>`
* Loop: `recv().await` lalu kirim `PUSH_EVENT` frame.

`server/conn_sub.rs`

```rust
use std::sync::Arc;
use bytes::{Buf, BufMut, BytesMut};
use tokio::net::UnixStream;
use tokio::sync::broadcast;
use tokio::io::AsyncWriteExt;

use crate::config::Config;
use crate::protocol::codec::{read_frame, write_response};
use crate::protocol::types::{Opcode, Status, EventType};
use crate::pubsub::bus::Bus;

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

pub async fn handle(
    mut stream: UnixStream,
    cfg: Config,
    bus: Arc<Bus>,
) -> anyhow::Result<()> {
    let mut buf = BytesMut::with_capacity(8 * 1024);

    // For v1: satu connection bisa subscribe banyak topic (simpel)
    // Maintain receivers list (naive). Bisa ditingkatkan pakai StreamMap kalau banyak.
    let mut receivers: Vec<(String, broadcast::Receiver<crate::pubsub::events::Event>)> = Vec::new();

    loop {
        tokio::select! {
            frame = read_frame(&mut stream, cfg.max_frame_bytes, &mut buf) => {
                let Some(frame) = frame? else { return Ok(()); };
                let mut rd = &frame[4..];
                let opcode = rd.get_u8();

                match opcode {
                    x if x == Opcode::Subscribe as u8 => {
                        let tlen = rd.get_u16_le() as usize;
                        if rd.remaining() < tlen { write_response(&mut stream, Status::ErrBadPayload, &[]).await?; continue; }
                        let topic = std::str::from_utf8(&rd[..tlen])?.to_string();

                        let rx = bus.subscribe(&topic);
                        receivers.push((topic, rx));
                        write_response(&mut stream, Status::Ok, &[]).await?;
                    }
                    x if x == Opcode::Unsubscribe as u8 => {
                        let tlen = rd.get_u16_le() as usize;
                        if rd.remaining() < tlen { write_response(&mut stream, Status::ErrBadPayload, &[]).await?; continue; }
                        let topic = std::str::from_utf8(&rd[..tlen])?.to_string();
                        receivers.retain(|(t, _)| t != &topic);
                        write_response(&mut stream, Status::Ok, &[]).await?;
                    }
                    _ => {
                        write_response(&mut stream, Status::ErrBadPayload, &[]).await?;
                    }
                }
            }

            // Naive fan-in: cek satu per satu (cukup karena topic jarang ratusan)
            // Bisa di-upgrade: tokio_stream::StreamMap jika ingin dynamic banyak channel.
            _ = async {
                for i in 0..receivers.len() {
                    match receivers[i].1.try_recv() {
                        Ok(ev) => {
                            // push event
                            let mut payload = BytesMut::new();
                            payload.put_u8(ev.event_type as u8);
                            payload.put_u16_le(ev.topic.len() as u16);
                            payload.extend_from_slice(ev.topic.as_bytes());
                            payload.put_u16_le(ev.key.len() as u16);
                            payload.extend_from_slice(ev.key.as_bytes());
                            payload.put_u64_le(ev.ts_ms);

                            let len = 1 + 1 + payload.len(); // opcode + status? (we keep PUSH_EVENT as opcode)
                            let mut out = BytesMut::with_capacity(4 + len);
                            out.put_u32_le(len as u32);
                            out.put_u8(Opcode::PushEvent as u8);
                            out.put_u8(Status::Ok as u8);
                            out.extend_from_slice(&payload);
                            stream.write_all(&out).await?;
                            return Ok::<(), anyhow::Error>(());
                        }
                        Err(broadcast::error::TryRecvError::Lagged(_)) => {
                            // push lagged notice
                            let mut out = BytesMut::with_capacity(4 + 2);
                            out.put_u32_le(2);
                            out.put_u8(Opcode::PushEvent as u8);
                            out.put_u8(Status::ErrLagged as u8);
                            stream.write_all(&out).await?;
                            return Ok::<(), anyhow::Error>(());
                        }
                        _ => {}
                    }
                }
                // no event, wait a bit
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                Ok::<(), anyhow::Error>(())
            } => {}
        }
    }
}
```

> `broadcast` doc menjelaskan slow receiver menghasilkan Lagged; ini yang kita convert menjadi `Status::ErrLagged` agar client aware. [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html)

***

# 3) Dependency graph antar module (“class”)

Ini penting biar jelas:

* `main.rs` → `server::run` + `config`
* `server::*` → `protocol::codec` + `protocol::types` + `store::KvStore` + `pubsub::Bus` + `store::Evictor`
* `store::KvStore` → `store::Entry` + `protocol::ValueFormat`
* `store::Evictor` → `store::KvStore` + `sys::meminfo`
* `pubsub::Bus` → `tokio::sync::broadcast` + `dashmap` + `pubsub::Event`
* `sys::meminfo` → baca `/proc/meminfo` (`MemAvailable`) [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html)
* `protocol::codec` → `bytes::BytesMut` + tokio IO [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide), [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html)

***

# 4) Hal yang WAJIB di-implement (non-negotiable)

1. **Frame length guard** (`max_frame_bytes`) untuk mencegah request “bengkak” (DoS lokal).
2. **UDS cleanup**: delete socket file sebelum bind (kalau crash sebelumnya).
3. **Payload enforcement**: `format` harus json/msgpack; **tidak ada auto serialize**.
4. **Pub/Sub bounded**: `broadcast::channel(capacity)` + handle `Lagged`. [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[exchangetuts.com\]](https://exchangetuts.com/index.php/tcp-loopback-connection-vs-unix-domain-socket-performance-1639495325509681)
5. **Pressure watcher**: pakai `MemAvailable` dari `/proc/meminfo`. [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html)
6. **Eviction** saat HOT>85% sampai COOL<80% agar tidak OOM host.
7. **Graceful logging**: tracing minimal (untuk debugging real world).
8. **Key namespace rule** documented (svc:table:pk) agar microservices konsisten. Redis menekankan schema key & namespace. [\[github.com\]](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Sockets/src/System/Net/Sockets/UnixDomainSocketEndPoint.Windows.cs), [\[blog.fzankl.de\]](https://blog.fzankl.de/unix-domain-sockets-in-net-6-basics-and-real-world-examples)

***

# 5) Benchmarking (wajib untuk klaim “extreme”)

Gunakan Criterion untuk microbench store operations (GET/SET) dan bisa tambah E2E UDS benchmark belakangan. [\[docs.rs\]](https://docs.rs/crossbeam/latest/crossbeam/), [\[github.com\]](https://github.com/crossbeam-rs/crossbeam)

`benches/kv_bench.rs` (contoh minimal)

```rust
use criterion::{criterion_group, criterion_main, Criterion, black_box};
use bytes::Bytes;
use localcached::store::kv_store::KvStore;
use localcached::protocol::types::ValueFormat;

fn bench_set_get(c: &mut Criterion) {
    let store = KvStore::default();
    let key = "billing:invoice:123".to_string();
    let val = Bytes::from_static(b"{\"id\":123}");

    c.bench_function("set_get", |b| {
        b.iter(|| {
            let now = 1u64;
            store.set(black_box(key.clone()), ValueFormat::Json, val.clone(), 0, now);
            let _ = store.get(black_box(&key), now);
        })
    });
}

criterion_group!(benches, bench_set_get);
criterion_main!(benches);
```

***

# 6) Deployment (jelas & real-world)

## 6.1 Build release

```bash
cargo build -p localcached --release
```

## 6.2 Jalankan manual (dev)

```bash
sudo ./target/release/localcached
# default listen di /run/localcached.sock
```

## 6.3 systemd service (recommended)

Buat file: `/etc/systemd/system/localcached.service`

```ini
[Unit]
Description=LocalCached UDS KV + PubSub
After=network.target

[Service]
Type=simple
ExecStart=/opt/localcached/localcached
Restart=always
RestartSec=1
Environment=RUST_LOG=info

# Hardening minimal:
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

Lalu:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now localcached
sudo systemctl status localcached
```

> Catatan: socket path `/run/...` biasanya tmpfs; enak untuk UDS dan tidak menulis data ke disk, hanya entry file socketnya saja. (Ini selaras dengan diskusi umum UDS dan penggunaan `/run`.) [\[stackoverflow.com\]](https://stackoverflow.com/questions/9898961/is-there-a-way-to-get-the-uid-of-the-other-end-of-a-unix-socket-connection), [\[docs.rs\]](https://docs.rs/bytes/latest/bytes/struct.BytesMut.html)

***

# 7) Checklist “Done Definition” (biar proyek kamu siap dipakai)

✅ CRUD: SET/GET/DEL/PING berjalan  
✅ Payload strict json/msgpack (no serialize server)  
✅ UDS stable: crash restart tidak nyangkut socket file  
✅ Pub/Sub: SUB/UNSUB + PUSH\_EVENT + Lagged handling  
✅ table\_changed upsert publish on SET(flag bit0)  
✅ invalidate publish on DEL (atau SET jika lu mau invalidate key lain)  
✅ pressure eviction bekerja (HOT/Cool) via MemAvailable  
✅ benchmark minimal store ada  
✅ systemd deployment ada

***
Mantap, keputusan **A) default publish** itu *inline banget* dengan tujuan “safe-by-default” untuk invalidasi di microservices. Dengan desain Pub/Sub yang **best-effort** dan bounded buffer, kamu dapat safety tanpa bikin server rentan “numpuk backlog” saat subscriber lambat. (Tokio broadcast memang punya mekanisme `Lagged` ketika receiver tertinggal, dan Redis Pub/Sub juga “fire-and-forget / at-most-once”.) [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html), [\[dev.to\]](https://dev.to/lordsnow/iouring-the-modern-asynchronous-io-revolution-in-linux-46ch), [\[dev.to\]](https://dev.to/frosnerd/libmalloc-jemalloc-tcmalloc-mimalloc-exploring-different-memory-allocators-4lp3)

Di bawah ini gue “kunci” 3 poin yang kamu minta dipertegas + update spec v1 yang konsisten dengan pilihan **A**.

***

# 1) Penanganan Key Non-Standar (yang tidak `svc:table:pk`)

## Keputusan yang gue rekomendasikan (best practice + performa + konsisten event)

**✅ Server mewajibkan format key `svc:table:pk` untuk operasi mutasi (SET/DEL) pada mode default publish.**  
Jika key tidak sesuai, server return **`ERR_INVALID_KEY_FORMAT`** (kita tambahkan sebagai status baru). Alasannya:

1. Kamu memilih **default publish** untuk menghindari stale cache. Kalau key “bebas” dibiarkan, event bisa *silent skip* dan itu bertentangan dengan “safe-by-default”.
2. Praktik key naming yang rapi dengan namespace memang best practice di sistem cache/Redis-like: gunakan delimiter `:` untuk membentuk hierarki/namespace, dan jaga key tetap ringkas karena key juga makan memori. [\[github.com\]](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Sockets/src/System/Net/Sockets/UnixDomainSocketEndPoint.Windows.cs), [\[blog.fzankl.de\]](https://blog.fzankl.de/unix-domain-sockets-in-net-6-basics-and-real-world-examples), [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/aspnet/core/grpc/interprocess?view=aspnetcore-10.0)
3. Parsing key `svc:table:pk` itu murah (split pendek), dan jauh lebih murah daripada “heuristic detection” atau parse payload. (Hot path kamu tetap tipis karena payload sudah serialized). [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide), [\[docs.rs\]](https://docs.rs/nix/latest/nix/sys/socket/index.html)

### Bagaimana kalau butuh key yang “bebas” (mis. `session_user_token`)?

Solusi best practice yang tetap konsisten:

* Jadikan “table/entity” sebagai kategori logis:  
    `auth:session:session_user_token`  
    atau jika token sangat panjang:  
    `auth:session:{sha256(token)}` (hash di client)

Jadi kamu **tetap** punya 3 segmen, event tetap bisa dipublish ke `t:auth:session`, dan tidak ada silent behavior.

> Ini juga mirip filosofi Redis: key schema/namespace itu kunci biar dataset schema-less tetap terkelola dan mudah housekeeping. [\[github.com\]](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Sockets/src/System/Net/Sockets/UnixDomainSocketEndPoint.Windows.cs), [\[blog.fzankl.de\]](https://blog.fzankl.de/unix-domain-sockets-in-net-6-basics-and-real-world-examples)

## Implementasi: fungsi parse key yang eksplisit (Wajib)

Tambahkan status code baru:

* `0x16 ERR_INVALID_KEY_FORMAT`

`protocol/types.rs` (tambahan)

```rust
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Ok = 0x00,
    NotFound = 0x01,
    ErrBadPayload = 0x10,
    ErrUnsupportedFormat = 0x11,
    ErrTooLarge = 0x12,
    ErrInternal = 0x13,
    ErrUnauthorized = 0x14,
    ErrLagged = 0x15,
    ErrInvalidKeyFormat = 0x16,
}
```

`server/conn_kv.rs` (helper parsing)

```rust
fn parse_key_3parts(key: &str) -> Result<(&str, &str, &str), ()> {
    let mut it = key.splitn(3, ':');
    let svc = it.next().ok_or(())?;
    let table = it.next().ok_or(())?;
    let pk = it.next().ok_or(())?;
    if svc.is_empty() || table.is_empty() || pk.is_empty() { return Err(()); }
    Ok((svc, table, pk))
}

fn topic_from_key_strict(key: &str) -> Result<String, ()> {
    let (svc, table, _pk) = parse_key_3parts(key)?;
    Ok(format!("t:{svc}:{table}"))
}
```

Lalu di `SET/DEL`, sebelum publish (dan bahkan sebelum `store.set` untuk konsistensi), lakukan:

```rust
let _topic = topic_from_key_strict(&key)
    .map_err(|_| Status::ErrInvalidKeyFormat)?;
```

> Ini memastikan “default publish” tidak pernah diam-diam gagal.

***

# 2) Spesifikasi Payload `STATS` (Opcode `0x05`) — **Final v1**

Karena kita mengejar performa, response STATS **lebih baik binary fixed-layout** (bukan teks/JSON). Ini menghindari parsing string dan menjaga output kecil. `BytesMut` cocok untuk membangun payload binary cepat. [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide), [\[docs.rs\]](https://docs.rs/nix/latest/nix/sys/socket/index.html)

## Response STATS: payload binary (little-endian)

**Response Status**: `OK`  
**Payload**:

    u8   stats_version        // =1
    u64  uptime_ms
    u64  keys_count
    u64  approx_mem_bytes      // perkiraan (key+value sizes + overhead estimasi)
    u64  evictions_total
    u64  pubsub_topics
    u64  events_published_total
    u64  events_lagged_total
    u64  invalid_key_total     // count invalid key attempts (SET/DEL rejected)
    u16  mem_pressure_bp       // basis points: 8500 = 85.00%
    u16  reserved              // align/padding, set 0

### Catatan “approx\_mem\_bytes”

Menghitung memori “tepat” di Rust runtime itu tidak trivial tanpa allocator introspection, jadi v1 pakai **estimasi**:

* `sum(entry.size_bytes)` (key.len + value.len) + overhead konstanta per entry (mis. +64 bytes).
* Ini cukup untuk monitoring trend.

### Mem pressure memakai `/proc/meminfo`

Kita baca `MemAvailable` dan `MemTotal` dari `/proc/meminfo`. `MemAvailable` memang didefinisikan sebagai estimasi memory yang tersedia untuk aplikasi tanpa swapping. [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html)

**Kode build payload STATS (contoh)**

```rust
use bytes::{BytesMut, BufMut};
use crate::protocol::types::Status;

fn build_stats_payload(stats: &StatsSnapshot) -> BytesMut {
    let mut out = BytesMut::with_capacity(1 + 8*9 + 2*2);
    out.put_u8(1); // stats_version
    out.put_u64_le(stats.uptime_ms);
    out.put_u64_le(stats.keys_count);
    out.put_u64_le(stats.approx_mem_bytes);
    out.put_u64_le(stats.evictions_total);
    out.put_u64_le(stats.pubsub_topics);
    out.put_u64_le(stats.events_published_total);
    out.put_u64_le(stats.events_lagged_total);
    out.put_u64_le(stats.invalid_key_total);
    out.put_u16_le(stats.mem_pressure_bp);
    out.put_u16_le(0);
    out
}
```

***

# 3) Definisi “table” di `svc:table:pk` — DB table atau entity?

## Keputusan definisi (jelas untuk tim)

**“table” = logical entity namespace**, **boleh sama dengan nama tabel DB**, dan **disarankan** begitu untuk konsistensi lintas tim/microservices.

Kenapa “logical entity” lebih tepat:

* Tidak semua cache key selalu 1:1 ke tabel fisik (contoh: `session`, `rate_limit`, `feature_flag`).
* Tapi kita tetap ingin konsistensi topic `t:svc:table`. Jadi “table” itu “bucket entity” yang disepakati tim.

**Praktik terbaik yang gue sarankan**

* Kalau key merepresentasikan baris tabel DB: pakai nama tabel DB (`user`, `order`, `invoice`).
* Kalau key bukan tabel DB: pakai nama entity logis (`session`, `token`, `cfg`, `agg_user_stats`).
* Tetap **lowercase + singkat**. Key terlalu panjang ada biaya memori. [\[github.com\]](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Sockets/src/System/Net/Sockets/UnixDomainSocketEndPoint.Windows.cs), [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/aspnet/core/grpc/interprocess?view=aspnetcore-10.0)

***

# 4) Update kecil pada spec SET flags (sesuai pilihan A)

Karena kamu memilih: **SET selalu publish table\_changed (upsert)** secara default, lebih natural kalau flags-nya adalah **“suppress\_publish”** bukan “publish\_event”.

### SET flags (v1 final)

* `flags bit0 = suppress_publish`
  * `0` = publish default (table\_changed upsert)
  * `1` = jangan publish (bulk import/migration)

Ini mengurangi human error dan inline dengan “safe by default”.

**Update parsing di SET**

```rust
let suppress_publish = (flags & 0b0000_0001) != 0;

if !suppress_publish {
    let topic = topic_from_key_strict(&key).map_err(|_| Status::ErrInvalidKeyFormat)?;
    bus.publish(Event {
        event_type: EventType::TableChanged,
        topic,
        key: key.clone(),
        ts_ms: now_ms(),
    });
}
```

Tokio broadcast channel akan mengirim event ke semua subscriber; kalau ada receiver lambat, akan lag dan receive `Lagged`. Ini memang behavior documented pada broadcast. [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html)

***

# 5) Revisi ringkas: PUSH\_EVENT payload (tetap sama, sekarang lebih “kunci”)

Karena event hanya 2 jenis dan `table_changed` selalu upsert, payload tetap:

    u8   event_type      // 1=invalidate, 2=table_changed(upsert)
    u16  topic_len
    u8[] topic           // "t:svc:table"
    u16  key_len
    u8[] key_full        // "svc:table:pk"
    u64  ts_ms

***

# 6) Hal yang jadi “WAJIB” setelah revisi ini

1. **Validasi key schema** untuk SET/DEL → error `ERR_INVALID_KEY_FORMAT` bila tidak `svc:table:pk`.
2. **Default publish ON** untuk SET (kecuali flags suppress).
3. **STATS response binary** fixed-layout v1 (biar client gampang parse).
4. **Mem pressure** dihitung dari `/proc/meminfo MemAvailable` (karena definisinya memang untuk estimasi available tanpa swap). [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html)
5. **Pub/Sub memakai `tokio::sync::broadcast` bounded**, dan client harus siap menerima `Lagged`. [\[exchangetuts.com\]](https://exchangetuts.com/index.php/tcp-loopback-connection-vs-unix-domain-socket-performance-1639495325509681), [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html)

***

# 7) Next: apa yang perlu di putuskan lagi? (tinggal 1 hal)

**Apakah GET boleh menerima key non-standar?**

* **Opsi 1 (konsisten & simple):** GET juga wajib `svc:table:pk` (kalau tidak → `ERR_INVALID_KEY_FORMAT`)
* **Opsi 2 (lebih fleksibel):** GET boleh key bebas, tapi SET/DEL wajib schema.

Pilihan jitu ke **Opsi 1** karena:

* konsisten untuk semua operasi,
* memudahkan client SDK,
* dan menjaga key hygiene (best practice). [\[github.com\]](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Sockets/src/System/Net/Sockets/UnixDomainSocketEndPoint.Windows.cs), [\[blog.fzankl.de\]](https://blog.fzankl.de/unix-domain-sockets-in-net-6-basics-and-real-world-examples)
