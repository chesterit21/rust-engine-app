# 🎯 Prompt Engineering Templates untuk Generate Dataset

## ⚠️ PENTING: Token Constraints

**Target Model**: Qwen3-0.6B (fine-tuning on Google Colab T4 16GB)

**Token Limits per Row Dataset:**

| Min | Ideal | Max |
|-----|-------|-----|
| 2,000 tokens | 4,000-8,000 tokens | 20,000 tokens |

**Jangan lebih dari 20K tokens per row untuk hindari OOM!**

---

## Cara Pakai

1. Copy prompt ke ChatGPT/Gemini/Deepseek/dll
2. Sesuaikan variabel `[PLACEHOLDER]` sesuai kebutuhan
3. Generate → dapat output JSONL
4. Gabungkan ke `train_data.jsonl`

---

## 🔧 TEMPLATE 1: Generate System-Level Dataset

**Prompt untuk AI Besar:**

```
Kamu adalah Dataset Generator Expert. Tugasmu adalah menghasilkan training data dalam format JSONL untuk fine-tuning model AI (Qwen3-0.6B) yang akan menjadi Expert Agent Planner.

### TOKEN CONSTRAINTS (PENTING!):
- Target: 2,000 - 8,000 tokens per sample (ideal: 4,000-6,000)
- JANGAN lebih dari 20,000 tokens per sample (akan OOM saat training)
- Context + Think + Response harus proporsional

### TASK:
Generate 10 training samples untuk SYSTEM-LEVEL planning.

### FORMAT OUTPUT SETIAP SAMPLE:
Setiap sample harus dalam format JSONL (1 JSON per line):

{"messages": [{"role": "system", "content": "[SYSTEM_PROMPT]\n\n### CONTEXT:\n[CONTEXT_TENTANG_SISTEM]"}, {"role": "user", "content": "[USER_REQUEST_VAGUE_ATAU_DETAILED]"}, {"role": "assistant", "content": "<think>[REASONING_STEP_BY_STEP]</think>\n[JSON_RESPONSE]"}]}

### SYSTEM_PROMPT (KONSISTEN):
"Kamu adalah Expert Agent Planner. Baca context yang diberikan dengan teliti. Analisis request user. Respond dengan JSON yang berisi module breakdown. Gunakan <think> untuk menunjukkan reasoning process."

### REQUIREMENTS:
1. CONTEXT harus berisi informasi lengkap tentang sistem (modules, features, dependencies) - target 1000-3000 tokens
2. USER REQUEST bervariasi (vague/medium/detailed)
3. THINK harus reasoning step-by-step - target 500-1000 tokens
4. JSON RESPONSE lengkap dengan semua modules - target 500-2000 tokens
5. JSON RESPONSE format:
   {
     "definition_domain": "module",
     "definition_scope": [
       {"name": "Module Name", "priority": 1, "reason": "why needed"}
     ]
   }

### DOMAIN SYSTEMS TO COVER:
1. HR System (Human Resource)
2. E-Commerce/Marketplace
3. CRM (Customer Relationship Management)
4. LMS (Learning Management System)
5. POS (Point of Sale)
6. Inventory Management
7. Project Management
8. Hospital/Clinic System
9. School/University System
10. Fintech/Banking

### OUTPUT:
Generate 10 samples (1 per domain) dalam format JSONL. Pastikan setiap line adalah valid JSON dan dalam range 2K-8K tokens.
```

---

## 🔧 TEMPLATE 2: Generate Module-Level Dataset

**Prompt untuk AI Besar:**

```
Kamu adalah Dataset Generator Expert. Tugasmu adalah menghasilkan training data dalam format JSONL untuk fine-tuning model AI yang akan menjadi Expert Agent Planner.

### TASK:
Generate 10 training samples untuk MODULE-LEVEL planning.

### FORMAT OUTPUT:
{"messages": [{"role": "system", "content": "[SYSTEM_PROMPT]\n\n### CONTEXT:\n[CONTEXT_TENTANG_MODULE]"}, {"role": "user", "content": "[USER_REQUEST]"}, {"role": "assistant", "content": "<think>[REASONING]</think>\n[JSON_RESPONSE]"}]}

### SYSTEM_PROMPT (KONSISTEN):
"Kamu adalah Expert Agent Planner. Baca context module yang diberikan. Analisis request user untuk detail module. Respond dengan JSON yang berisi feature breakdown."

### REQUIREMENTS:
1. CONTEXT harus berisi: nama module, daftar features dalam module, validation rules, dependencies
2. USER REQUEST: "detail kan module X", "breakdown fitur module X", etc.
3. THINK: reasoning tentang fitur mana yang relevan dengan request
4. JSON RESPONSE format:
   {
     "definition_domain": "feature",
     "parent_module": "Module Name",
     "definition_scope": [
       {"name": "Feature Name", "priority": 1, "entities": ["table1", "table2"], "validation": {...}}
     ]
   }

### MODULES TO COVER:
1. User Management (auth, roles, permissions)
2. Product Catalog (CRUD produk, kategori, gambar)
3. Shopping Cart (add, remove, quantity)
4. Order Management (checkout, status, tracking)
5. Payment Gateway (integration, verification)
6. Employee Management (CRUD, department, position)
7. Attendance Management (clock in/out, leave)
8. Payroll Management (salary, deduction, slip)
9. File Upload (validation, storage, retrieval)
10. Notification System (push, email, in-app)

### OUTPUT:
Generate 10 samples (1 per module) dalam format JSONL.
```

---

## 🔧 TEMPLATE 3: Generate Feature-Level Dataset (dengan Tech Stack Context)

**Prompt untuk AI Besar:**

```
Kamu adalah Dataset Generator Expert. Tugasmu adalah menghasilkan training data dalam format JSONL untuk fine-tuning model AI yang akan menjadi Expert Agent Planner.

### TASK:
Generate 10 training samples untuk FEATURE-LEVEL planning dengan TECH STACK CONTEXT.

### FORMAT OUTPUT:
{"messages": [{"role": "system", "content": "[SYSTEM_PROMPT]\n\n### CONTEXT:\n[CONTEXT_TENTANG_BEST_PRACTICES_TECH_STACK]"}, {"role": "user", "content": "[USER_REQUEST_DENGAN_TECH_STACK]"}, {"role": "assistant", "content": "<think>[REASONING]</think>\n[JSON_RESPONSE]"}]}

### SYSTEM_PROMPT (KONSISTEN):
"Kamu adalah Expert Agent Planner. Baca context best practices yang diberikan. Analisis request user dengan mempertimbangkan tech stack yang disebutkan. Respond dengan JSON yang berisi implementation tasks."

### REQUIREMENTS:
1. CONTEXT harus berisi: best practices untuk tech stack tersebut (library, validation rules, security, file naming, path convention)
2. USER REQUEST harus menyertakan tech stack: "implementasi upload file pakai Node.js + React"
3. THINK: reasoning berdasarkan context → pilih library → tentukan validasi
4. JSON RESPONSE format:
   {
     "definition_domain": "task",
     "parent_feature": "Feature Name",
     "tech_stack": {"frontend": "React", "backend": "Node.js", "database": "PostgreSQL"},
     "definition_scope": [
       {
         "task_name": "Frontend Validation",
         "library": "react-dropzone",
         "validation_rules": ["extension: jpg,png", "max_size: 2MB"],
         "implementation_notes": "..."
       }
     ]
   }

### FEATURE + TECH COMBINATIONS TO COVER:
1. File Upload + Node.js/React
2. File Upload + Python/FastAPI
3. Authentication + Node.js/Express
4. Authentication + Python/Django
5. Payment Integration + Node.js
6. CRUD Operations + Laravel/PHP
7. Real-time Chat + Node.js/Socket.io
8. Search Feature + Elasticsearch
9. Report Export + Python/Pandas
10. Email Sending + Node.js/Nodemailer

### OUTPUT:
Generate 10 samples (1 per combination) dalam format JSONL.
```

---

## 🔧 TEMPLATE 4: Generate Variasi User Request (Vague to Detailed)

**Prompt untuk AI Besar:**

```
Kamu adalah Dataset Generator Expert. Tugasmu adalah menghasilkan VARIASI USER REQUEST untuk satu topic yang sama.

### TASK:
Untuk setiap topic di bawah, generate 3 variasi user request: VAGUE, MEDIUM, DETAILED.

### FORMAT OUTPUT:
Untuk setiap topic, output 3 JSONL entries dengan context yang SAMA tapi user request yang BERBEDA tingkat detail-nya.

### TOPICS:
1. Membuat fitur upload gambar produk
2. Membuat sistem login user
3. Membuat fitur shopping cart
4. Membuat dashboard admin
5. Membuat sistem notifikasi

### EXAMPLE OUTPUT untuk topic "upload gambar produk":

Vague:
{"messages": [{"role": "system", "content": "...\n\n### CONTEXT:\n[context upload]"}, {"role": "user", "content": "buatkan fitur upload gambar"}, {"role": "assistant", "content": "<think>Request vague, tidak spesifik. Dari context, standard upload adalah...</think>\n{...}"}]}

Medium:
{"messages": [{"role": "system", "content": "...\n\n### CONTEXT:\n[context upload]"}, {"role": "user", "content": "buatkan fitur upload gambar produk, max 5 gambar"}, {"role": "assistant", "content": "<think>Request medium detail, ada constraint max 5. Dari context...</think>\n{...}"}]}

Detailed:
{"messages": [{"role": "system", "content": "...\n\n### CONTEXT:\n[context upload]"}, {"role": "user", "content": "buatkan fitur upload gambar produk marketplace, max 5 gambar, format jpg/png, max 2MB per file, dengan preview dan drag-drop"}, {"role": "assistant", "content": "<think>Request detailed dengan banyak constraint. Dari context, semua requirement sudah spesifik...</think>\n{...}"}]}

### OUTPUT:
Generate 15 samples total (3 variasi x 5 topics) dalam format JSONL.
```

---

## 🔧 TEMPLATE 5: Generate Context Documents

**Prompt untuk AI Besar:**

```
Kamu adalah Technical Documentation Expert. Tugasmu adalah membuat CONTEXT DOCUMENTS yang akan digunakan dalam training dataset.

### TASK:
Buat context document untuk [TOPIC] yang berisi:
1. Overview/Definition
2. Standard Components/Modules/Features
3. Best Practices
4. Validation Rules
5. Security Considerations
6. Common Pitfalls

### FORMAT OUTPUT:
Markdown document yang bisa di-inject ke system prompt.

### TOPICS TO GENERATE:
1. HR System Architecture
2. E-Commerce System Architecture
3. User Management Module
4. File Upload Feature (Node.js)
5. File Upload Feature (Python)
6. Authentication Best Practices
7. Payment Gateway Integration
8. Database Design Patterns

### EXAMPLE OUTPUT untuk "File Upload Feature (Node.js)":

## FILE UPLOAD FEATURE - NODE.JS

### Libraries
- multer: file handling middleware
- sharp: image processing
- uuid: filename generation

### Validation Rules
- Extension whitelist: .jpg, .jpeg, .png, .gif, .webp
- MIME type validation: check actual content, not just extension
- File size: 2MB client-side, 5MB server-side
- File count: configurable per use case

### File Naming Convention
- Format: {uuid}_{timestamp}.{ext}
- Example: a1b2c3d4_1705632000.jpg

### Storage Path
- Pattern: /uploads/{year}/{month}/{day}/{filename}
- Example: /uploads/2024/01/19/a1b2c3d4_1705632000.jpg

### Security
- Virus scanning dengan ClamAV
- Path traversal prevention
- Rate limiting: 20 uploads/user/hour
- Private storage (tidak public accessible)

### Error Handling
- File too large: 413 Payload Too Large
- Invalid type: 415 Unsupported Media Type
- Storage full: 507 Insufficient Storage

---

Generate context document untuk: [MASUKKAN_TOPIC_DISINI]
```

---

## 📋 Workflow Penggunaan

```
Step 1: Generate Context Documents (Template 5)
   ↓
Step 2: Generate System-Level Dataset (Template 1)
   ↓
Step 3: Generate Module-Level Dataset (Template 2)
   ↓
Step 4: Generate Feature-Level Dataset (Template 3)
   ↓
Step 5: Generate Request Variations (Template 4)
   ↓
Step 6: Combine all JSONL → train_data.jsonl
   ↓
Step 7: Train model!
```

---

## 💡 Tips

1. **Gunakan model berbeda untuk variasi**: ChatGPT untuk satu batch, Gemini untuk batch lain
2. **Validate JSONL**: Pastikan setiap line adalah valid JSON sebelum gabung
3. **Quality check**: Review beberapa samples untuk pastikan reasoning dan JSON konsisten
4. **Iterative**: Mulai dengan 50 samples, test, iterate
