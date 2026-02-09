# Svelte + Vite

This template should help get you started developing with Svelte in Vite.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode).

## Need an official Svelte framework?

Check out [SvelteKit](https://github.com/sveltejs/kit#readme), which is also powered by Vite. Deploy anywhere with its serverless-first approach and adapt to various platforms, with out of the box support for TypeScript, SCSS, and Less, and easily-added support for mdsvex, GraphQL, PostCSS, Tailwind CSS, and more.

## Technical considerations

**Why use this over SvelteKit?**

- It brings its own routing solution which might not be preferable for some users.
- It is first and foremost a framework that just happens to use Vite under the hood, not a Vite app.

This template contains as little as possible to get started with Vite + Svelte, while taking into account the developer experience with regards to HMR and intellisense. It demonstrates capabilities on par with the other `create-vite` templates and is a good starting point for beginners dipping their toes into a Vite + Svelte project.

Should you later need the extended capabilities and extensibility provided by SvelteKit, the template has been structured similarly to SvelteKit so that it is easy to migrate.

**Why include `.vscode/extensions.json`?**

Other templates indirectly recommend extensions via the README, but this file allows VS Code to prompt the user to install the recommended extension upon opening the project.

**Why enable `checkJs` in the JS template?**

It is likely that most cases of changing variable types in runtime are likely to be accidental, rather than deliberate. This provides advanced typechecking out of the box. Should you like to take advantage of the dynamically-typed nature of JavaScript, it is trivial to change the configuration.

**Why is HMR not preserving my local component state?**

HMR state preservation comes with a number of gotchas! It has been disabled by default in both `svelte-hmr` and `@sveltejs/vite-plugin-svelte` due to its often surprising behavior. You can read the details [here](https://github.com/sveltejs/svelte-hmr/tree/master/packages/svelte-hmr#preservation-of-local-state).

If you have state that's important to retain within a component, consider creating an external store which would not be replaced by HMR.

```js
// store.js
// An extremely simple external store
import { writable } from 'svelte/store'
export default writable(0)
```

### DEPLOY

```
docker build -t svelte-app:latest .
```

Good question bro! Ada 2 cara:

---

## **Cara 1: Rebuild & Replace Container** (Simple, recommended)

Setiap kali ada update:

```bash
# 1. Build ulang dist
npm run build

# 2. Stop & remove container lama
docker stop svelte-app
docker rm svelte-app

# 3. Rebuild image (overwrite yang lama)
docker build -t tradex-app:latest .

# 4. Run container baru
docker run -d --name svelte-app -p 55660:55660 tradex-app:latest
```

---

## **Cara 2: Pake Volume Mount** (Development mode)

Kalo mau **live update** tanpa rebuild (tapi ukuran image tetep kecil):

**Ubah Dockerfile jadi:**

```dockerfile
FROM nginx:alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 55660
CMD ["nginx", "-g", "daemon off;"]
```

**Run dengan mount dist folder:**

```bash
docker run -d \
  --name tradex-app \
  -p 55660:55660 \
  -v $(pwd)/dist:/usr/share/nginx/html \
  tradex-app:latest
```

Setiap `npm run build`, langsung keupdate otomatis (refresh browser aja).

---

Aha! Ketemu masalahnya bro!

**Error:** `host not found in upstream "host.docker.internal"`

`host.docker.internal` itu cara Docker Desktop (Windows/Mac) buat akses host machine. Di **Linux** gak work!

---

## **Solusi: Ganti jadi IP host atau Docker bridge**

**Update `nginx.conf`:**

```nginx
server {
    listen 55660;
    server_name localhost;

    root /usr/share/nginx/html;
    index index.html;

    # Proxy API calls ke backend - GANTI INI
    location /api {
        proxy_pass http://172.17.0.1:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # Serve static files
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

**Penjelasan:**

- `172.17.0.1` = IP default Docker bridge di Linux, pointing ke host machine
- Ini biar container svelte bisa akses backend yang jalan di host

---

**Rebuild & run lagi:**

```bash
# Rebuild image
docker build -t tradex-app:latest .

# Remove container lama
docker rm tradex-app

# Run lagi
docker run -d --name tradex-app -p 55660:55660 tradex-app:latest

# Cek logs
docker logs tradex-app
```

#### Jika ada Update lagi

```bash
# 1. Build ulang dist
npm run build

# 2. Stop & remove container lama
docker stop tradex-app
docker rm tradex-app

# 3. Rebuild image (overwrite yang lama)
docker build -t tradex-app:latest .

# 4. Run container baru
docker run -d --name tradex-app -p 55660:55660 tradex-app:latest
```
