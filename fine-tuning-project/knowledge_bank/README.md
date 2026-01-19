# 📚 Knowledge Bank

Bank pengetahuan untuk RAG system. Semua context documents disimpan di sini.

## Struktur Folder

```
knowledge_bank/
├── systems/        ← System architecture (HR, E-Commerce, CRM, dll)
├── modules/        ← Module specs (User Management, Payment, dll)
├── features/       ← Feature details (File Upload, Auth, dll)
├── validations/    ← Validation rules per tech stack
└── tech_stacks/    ← Tech stack best practices
```

## Cara Pakai

1. Generate context via AI besar (Template 1)
2. Save hasil ke folder yang sesuai
3. Gunakan sebagai RAG source untuk Agent Framework

## Naming Convention

- Lowercase dengan underscore
- Format: `{topic}_{tech}.md`
- Contoh: `file_upload_nodejs.md`, `auth_python_django.md`
