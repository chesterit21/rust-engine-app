# 🧠 REVISED: Context-Driven Agent Planner Dataset Strategy

## 🎯 The Cutting Edge Insight

**Problem dengan pendekatan lama:**

- Dataset melebar ke semua domain (business process, bahasa, ERD, server env)
- Qwen3-0.6B akan **catastrophic forgetting** dengan data terlalu banyak
- Model jadi **generalist yang lemah** bukan **specialist yang kuat**

**Solusi GENIUS:**

```
Model BUKAN belajar SEMUA knowledge
Model BELAJAR cara BACA CONTEXT dan RESPOND dengan STRUCTURED JSON
```

---

## 🔑 Key Formula

```
System Prompt = [CONSISTENT INSTRUCTION] + [RAG CONTEXT]

Model learns:
1. HOW to read context
2. HOW to extract relevant info from context
3. HOW to format response as JSON
4. HOW to think (via <think> tags)

Model does NOT memorize:
❌ All business processes
❌ All programming languages
❌ All validation rules
❌ All ERD patterns
```

---

## 📊 Dataset Structure (REVISED)

### **Format:**

```json
{
  "messages": [
    {
      "role": "system", 
      "content": "[CONSISTENT PROMPT]\n\n### CONTEXT:\n[RAG RESULT - domain knowledge injected here]"
    },
    {
      "role": "user", 
      "content": "[USER REQUEST - bisa vague atau detailed]"
    },
    {
      "role": "assistant", 
      "content": "<think>[REASONING PROCESS]</think>\n[JSON STRUCTURED RESPONSE]"
    }
  ]
}
```

### **Example - System Level:**

```json
{
  "messages": [
    {
      "role": "system",
      "content": "Kamu adalah Expert Agent Planner. Analisis context yang diberikan dan respond dengan JSON yang sesuai dengan Agent Framework.\n\n### CONTEXT:\nHR System adalah sistem untuk mengelola sumber daya manusia. Standard modules dalam HR System:\n- User Management: autentikasi, otorisasi, role\n- Employee Management: data karyawan, department, position\n- Attendance Management: absensi, izin, cuti\n- Payroll Management: gaji, tunjangan, potongan, slip gaji\n- Recruitment: lowongan, applicant tracking\n- Performance: KPI, review, appraisal"
    },
    {
      "role": "user",
      "content": "buatkan aplikasi HR simple saja, bisa manage karyawan, absensi, sama penggajian dasar"
    },
    {
      "role": "assistant",
      "content": "<think>User minta HR simple dengan 3 fitur: manage karyawan, absensi, penggajian dasar. Dari context, ini mapping ke: Employee Management, Attendance Management, Payroll Management. Karena simple, skip Recruitment dan Performance. Tapi User Management tetap wajib untuk auth.</think>\n{\"definition_domain\": \"module\", \"definition_scope\": [{\"name\": \"User Management\", \"priority\": 1, \"reason\": \"required for authentication\"}, {\"name\": \"Employee Management\", \"priority\": 2, \"reason\": \"manage karyawan\"}, {\"name\": \"Attendance Management\", \"priority\": 3, \"reason\": \"absensi\"}, {\"name\": \"Payroll Management\", \"priority\": 4, \"reason\": \"penggajian dasar\"}]}"
    }
  ]
}
```

### **Example - Module Level:**

```json
{
  "messages": [
    {
      "role": "system",
      "content": "Kamu adalah Expert Agent Planner...\n\n### CONTEXT:\nEmployee Management Module terdiri dari features:\n- CRUD Employee (create, read, update, delete)\n- Department Assignment\n- Position/Role Assignment\n- Employee Status (active, inactive, resigned)\n- Employee Documents\n- Emergency Contact\n\nValidation Rules:\n- NIK: 16 digit, unique\n- Email: format valid, unique\n- Phone: E.164 format\n- Photo: jpg/png, max 2MB"
    },
    {
      "role": "user",
      "content": "detail kan Employee Management module nya"
    },
    {
      "role": "assistant",
      "content": "<think>User minta detail Employee Management. Dari context ada 6 features. Untuk simple HR, fokus ke CRUD Employee, Department, Position, Status. Documents dan Emergency Contact optional.</think>\n{\"definition_domain\": \"feature\", \"parent_module\": \"Employee Management\", \"definition_scope\": [{\"name\": \"CRUD Employee\", \"priority\": 1, \"entities\": [\"employees\"], \"validation\": {\"nik\": \"16 digit, unique\", \"email\": \"format valid, unique\", \"phone\": \"E.164\"}}, {\"name\": \"Department Assignment\", \"priority\": 2}, {\"name\": \"Position Assignment\", \"priority\": 3}, {\"name\": \"Employee Status\", \"priority\": 4}]}"
    }
  ]
}
```

---

## 🏗️ Architecture Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   RAG/       │     │   Fine-tuned │     │   Agent      │
│   Vector DB  │────▶│   Qwen3-0.6B │────▶│   Framework  │
│   (Context)  │     │   (Reasoning)│     │   (Execute)  │
└──────────────┘     └──────────────┘     └──────────────┘
       │                     │                    │
  Domain Knowledge    Context Understanding   JSON → Action
  - System types      - Read context           - Parse response
  - Module patterns   - <think> reasoning      - Map to agents
  - Validation rules  - Structured JSON        - Execute tasks
```

---

## 📚 Context Database Structure (for RAG)

```
/contexts
├── /systems
│   ├── hr_system.md
│   ├── ecommerce_system.md
│   ├── crm_system.md
│   └── ...
├── /modules
│   ├── user_management.md
│   ├── employee_management.md
│   ├── product_catalog.md
│   └── ...
├── /features
│   ├── file_upload.md
│   ├── authentication.md
│   ├── payment_gateway.md
│   └── ...
└── /validation
    ├── nodejs_validation.md
    ├── python_validation.md
    └── ...
```

---

## 🎯 Benefits of This Approach

1. **Model stays small** - Qwen3-0.6B is enough
2. **No catastrophic forgetting** - knowledge is in RAG, not weights
3. **Infinitely scalable** - add new contexts without retraining
4. **Controllable output** - JSON for Agent Framework
5. **Reasoning visibility** - `<think>` tags for debugging

---

## 📊 Dataset Size Estimate

| Level | Samples Needed | Focus |
|-------|----------------|-------|
| System | 50-100 | Context reading + module extraction |
| Module | 100-200 | Context reading + feature extraction |
| Feature | 100-200 | Context reading + task extraction |
| Validation | 50-100 | Context reading + rules extraction |
| **Total** | **300-600** | |

**Much smaller than 1000+ without context approach!**

---

## 🚀 Next Steps

1. **Build Context Database** - Markdown files for each domain
2. **Create Dataset Generator** - Script that combines context + request + response
3. **Design JSON Schema** - Consistent output format for Agent Framework
4. **Train & Test** - Small batch first (50 samples), iterate

---

## ❓ Questions

1. JSON Schema untuk Agent Framework - sudah ada spec nya?
2. Context granularity - sedetail apa per file?
3. `<think>` tag format - ada requirement khusus?
