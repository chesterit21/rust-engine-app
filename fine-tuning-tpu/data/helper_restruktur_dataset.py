import json
import re

# Blueprint dari lo
module_definitions = {
    "product_upload": {
        "core_functionality": "Upload & manage product images",
        "required_validations": ["file_extension", "file_size", "file_count", "mime_type"],
        "tech_stack_dependencies": ["react-dropzone", "multer", "aws-sdk"],
        "compliance_requirements": ["gdpr", "accessibility"],
        "performance_metrics": ["upload_time", "image_load_time"]
    },
    "user_registration": {
        "core_functionality": "User signup with email verification",
        "required_validations": ["email_format", "password_strength", "phone_format"],
        "tech_stack_dependencies": ["nodemailer", "bcrypt", "redis"],
        "compliance_requirements": ["gdpr", "ccpa"],
        "performance_metrics": ["signup_completion_time"]
    }
}

# User request templates (bisa dikembangkan)
user_requests = {
    "product_upload": "Aku mau bikin fitur upload gambar produk untuk aplikasi marketplace. User bisa upload max 5 gambar per produk. Gambar harus muncul di halaman detail produk.",
    "user_registration": "Aku mau bikin sistem registrasi user untuk aplikasi marketplace. User harus bisa daftar dengan email dan password, lalu verifikasi email sebelum bisa login."
}

# System prompt template
SYSTEM_PROMPT = """Kamu adalah Expert Agent Planner. Tugas kamu adalah membuat detailed technical specification dalam bahasa manusia untuk developer. JANGAN generate code. Fokus pada:
1. Complete module definition
2. Frontend validation rules
3. Backend validation rules
4. Security best practices
5. Implementation procedures
6. Error handling scenarios
7. Performance considerations
8. Compliance requirements"""

def generate_planning_response(module_name, module_data):
    """Generate detailed planning response from blueprint"""
    
    # Mapping validation types to human readable format
    validation_mapping = {
        "file_extension": "File Extension: hanya .jpg, .jpeg, .png, .webp",
        "file_size": "File Size: max 2MB per file (client), 5MB (server)",
        "file_count": "File Count: max 5 gambar per produk",
        "mime_type": "MIME Type: validate actual content type matches extension",
        "email_format": "Email Format: RFC 5322 compliant validation",
        "password_strength": "Password Strength: min 8 chars, uppercase, number, symbol",
        "phone_format": "Phone Format: E.164 format validation (opsional)"
    }
    
    # Mapping compliance to human readable
    compliance_mapping = {
        "gdpr": "GDPR: explicit consent untuk data processing, right to delete",
        "ccpa": "CCPA: California Consumer Privacy Act compliance",
        "accessibility": "Accessibility: WCAG 2.1 compliance, screen reader support"
    }
    
    # Generate response based on module type
    if module_name == "product_upload":
        return f"""**MODULE DEFINITION: PRODUCT IMAGE UPLOAD**

**CORE FUNCTIONALITY**: {module_data['core_functionality']}

**REQUIRED VALIDATIONS**:
{chr(10).join(['- ' + validation_mapping[val] for val in module_data['required_validations']])}

**TECH STACK DEPENDENCIES**:
- Frontend: react-dropzone, react-hook-form
- Backend: multer, aws-sdk
- Database: PostgreSQL untuk metadata storage

**COMPLIANCE REQUIREMENTS**:
{chr(10).join(['- ' + compliance_mapping[req] for req in module_data['compliance_requirements']])}

**PERFORMANCE METRICS**:
- Upload time: <15 detik untuk 5 gambar di 4G
- Image load time: <1 detik (CDN cached)

**DETAILED IMPLEMENTATION GUIDE**:
1. **Frontend Validation**:
   • Real-time extension check (.jpg, .jpeg, .png, .webp only)
   • Client-side size validation (max 2MB)
   • File count counter (3/5 gambar)
   • Instant preview dengan URL.createObjectURL()

2. **Backend Validation**:
   • Re-check extensions dan MIME type
   • Server-side size limit 5MB
   • File naming: product_{{id}}_{{timestamp}}_{{random6}}.{{ext}}
   • Storage path: /marketplace/products/{{year}}/{{month}}/{{day}}/

3. **Security Measures**:
   • Virus scanning with ClamAV
   • Rate limiting: 20 uploads/user/hour
   • Path traversal protection

4. **Error Handling**:
   • Partial success handling (keep successful uploads)
   • Network interruption resume capability
   • Storage failure fallback mechanism

**ACCEPTANCE CRITERIA**:
□ All validation rules implemented di frontend dan backend
□ No security vulnerabilities in penetration testing
□ Performance metrics tercapai di real-world conditions"""
    
    elif module_name == "user_registration":
        return f"""**MODULE DEFINITION: USER REGISTRATION**

**CORE FUNCTIONALITY**: {module_data['core_functionality']}

**REQUIRED VALIDATIONS**:
{chr(10).join(['- ' + validation_mapping[val] for val in module_data['required_validations']])}

**TECH STACK DEPENDENCIES**:
- Email Service: nodemailer + SendGrid/Mailgun
- Password Hashing: bcrypt dengan cost factor 12
- Rate Limiting: redis untuk track failed attempts
- Database: PostgreSQL untuk user storage

**COMPLIANCE REQUIREMENTS**:
{chr(10).join(['- ' + compliance_mapping[req] for req in module_data['compliance_requirements']])}

**PERFORMANCE METRICS**:
- Signup completion time: <30 detik
- Email delivery time: <2 menit
- Verification link expiration: 24 jam

**DETAILED IMPLEMENTATION GUIDE**:
1. **Frontend Validation**:
   • Real-time email format validation
   • Password strength meter dengan visual feedback
   • Confirm password field matching
   • Form submission disable sampai semua valid

2. **Backend Validation**:
   • Transactional database insert (user + verification token)
   • Asynchronous email sending dengan queue system
   • Rate limiting: max 3 failed attempts per IP per minute
   • Password hashing sebelum save ke database

3. **Security Measures**:
   • Verification token dengan 32-character random string
   • Token expiration 24 jam
   • Prevent timing attacks dengan constant-time comparison
   • Sanitize all user inputs untuk prevent XSS

4. **Error Handling**:
   • Duplicate email: friendly error message
   • Email sending failure: retry mechanism + admin alert
   • Token expiration: automatic cleanup job
   • Account lockout after 10 failed login attempts

**ACCEPTANCE CRITERIA**:
□ User bisa complete signup flow dalam <30 detik
□ Email verification link valid hanya 24 jam
□ No security vulnerabilities in authentication flow
□ Full compliance dengan GDPR/CCPA requirements"""
    
    else:
        return "Module tidak dikenali. Silakan spesifikasikan module yang dibutuhkan."

# Generate JSONL file
with open('agent_planner_dataset.jsonl', 'w', encoding='utf-8') as f:
    for module_name, module_data in module_definitions.items():
        chat_template = {
            "messages": [
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": user_requests[module_name]},
                {"role": "assistant", "content": generate_planning_response(module_name, module_data)}
            ]
        }
        f.write(json.dumps(chat_template, ensure_ascii=False) + '\n')

print("✅ Dataset JSONL berhasil generate: agent_planner_dataset.jsonl")
print(f"📊 Total module: {len(module_definitions)}")