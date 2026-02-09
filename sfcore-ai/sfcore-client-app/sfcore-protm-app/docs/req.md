
bikinin Repository class, domain class, service, handler untuk masing-masing table di bawah ini.

TABLE architecture_patterns ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), parent_id TEXT NOT NULL, name TEXT NOT NULL, type TEXT NOT NULL CHECK(type IN ('BE', 'FE')), layer_rules TEXT, order_index INTEGER NOT NULL, naming_conventions TEXT, FOREIGN KEY (parent_id) REFERENCES architecture_patterns(id) )

TABLE attributes ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), entity_id TEXT NOT NULL,
name TEXT NOT NULL, -- "email", "password", "created_at"
data_type TEXT NOT NULL, -- "string", "integer", "boolean", "datetime"
 is_primary_key BOOLEAN DEFAULT 0, is_foreign_key BOOLEAN DEFAULT 0, is_nullable BOOLEAN DEFAULT 1, is_unique BOOLEAN DEFAULT 0,
max_length INTEGER, -- For string types
validation_rules TEXT, -- JSON array
business_rules TEXT, -- "Must be hashed", "Auto-generated UUID"
source_description TEXT, -- "User input via registration form"
order_index INTEGER, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE, UNIQUE(entity_id, name) )

TABLE entities ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), project_id TEXT NOT NULL,
name TEXT NOT NULL, -- "User", "Order", "Product"
table_name TEXT NOT NULL, -- "users", "orders" (DB table name)
description TEXT, is_aggregate_root BOOLEAN DEFAULT 0, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE, UNIQUE(project_id, table_name) )

TABLE entity_relationships ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
entity_id TEXT NOT NULL, -- Source entity
related_entity_id TEXT NOT NULL, -- Target entity
relationship_type TEXT CHECK(relationship_type IN ('one-to-one', 'one-to-many', 'many-to-many')),
foreign_key_attribute_id TEXT, -- Which attribute is the
FK description TEXT, -- "User has many Orders"
created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE, FOREIGN KEY (related_entity_id) REFERENCES entities(id) ON DELETE CASCADE, FOREIGN KEY (foreign_key_attribute_id) REFERENCES attributes(id) )

TABLE file_templates ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), layer_id TEXT NOT NULL, name TEXT NOT NULL, file_naming TEXT NOT NULL, class_naming TEXT NOT NULL, code_template TEXT, required_imports TEXT, required_methods TEXT, description TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (layer_id) REFERENCES pattern_layers(id) ON DELETE CASCADE )

TABLE flow_steps ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), task_id TEXT NOT NULL, order_index INTEGER NOT NULL,
type TEXT CHECK(type IN ('input', 'process', 'decision', 'output')) DEFAULT 'process', description TEXT NOT NULL, code_snippet TEXT, validation_rules TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE, UNIQUE(task_id, order_index) )

TABLE modules ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), project_id TEXT NOT NULL, name TEXT NOT NULL, description TEXT, order_index INTEGER, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE )

TABLE pattern_layers ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), pattern_id TEXT NOT NULL,
name TEXT NOT NULL, -- "domain", "application", "infrastructure"
path TEXT NOT NULL, -- "src/modules/{module}/domain" rules TEXT, -- Specific rules for this layer
order_index INTEGER NOT NULL, -- Display order
created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (pattern_id) REFERENCES architecture_patterns(id) ON DELETE CASCADE )

TABLE personas ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), project_id TEXT NOT NULL, name TEXT NOT NULL, description TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE )

TABLE projects ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), name TEXT NOT NULL, description TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, updated_at DATETIME DEFAULT CURRENT_TIMESTAMP )

TABLE stack_libraries ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), stack_id TEXT NOT NULL, name TEXT NOT NULL, npm_package TEXT, version TEXT, category TEXT, description TEXT, is_required BOOLEAN DEFAULT 0, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (stack_id) REFERENCES tech_stacks(id) ON DELETE CASCADE )

TABLE task_dependencies ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
task_id TEXT NOT NULL, -- Dependent task
depends_on_task_id TEXT NOT NULL, -- Required task (must complete first)
dependency_type TEXT CHECK(dependency_type IN ('blocks', 'requires', 'optional')) DEFAULT 'blocks', created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE, FOREIGN KEY (depends_on_task_id) REFERENCES tasks(id) ON DELETE CASCADE,
UNIQUE(task_id, depends_on_task_id),
CHECK(task_id != depends_on_task_id) -- Prevent self-dependency )

TABLE task_entity_usage ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), task_id TEXT NOT NULL, entity_id TEXT NOT NULL, operation TEXT CHECK(operation IN ('create', 'read', 'update', 'delete')),
attributes_used TEXT, -- JSON array of attribute IDs
created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE, FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE, UNIQUE(task_id, entity_id, operation) )

TABLE task_file_mappings ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), task_id TEXT NOT NULL, template_id TEXT NOT NULL, file_path TEXT NOT NULL, class_name TEXT NOT NULL, method_names TEXT, dependencies TEXT, implementation_notes TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE, FOREIGN KEY (template_id) REFERENCES file_templates(id) )

TABLE tasks ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), use_case_id TEXT NOT NULL, name TEXT NOT NULL, priority TEXT CHECK(priority IN ('P1', 'P2', 'P3')) DEFAULT 'P2', status TEXT CHECK(status IN ('todo', 'in_progress', 'done')) DEFAULT 'todo', description TEXT, validation_rules TEXT, order_index INTEGER, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, completed_at DATETIME, FOREIGN KEY (use_case_id) REFERENCES use_cases(id) ON DELETE CASCADE )

TABLE tech_stacks ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
name TEXT NOT NULL, -- "NestJS", "React + Vite", "Django"
type TEXT NOT NULL CHECK(type IN ('BE', 'FE')), language
TEXT NOT NULL, -- "TypeScript", "Python", "JavaScript"
description TEXT )

TABLE use_cases ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), user_story_id TEXT NOT NULL,
name TEXT NOT NULL, -- "User Registration", "Process Payment"
actor TEXT, -- "Guest User", "Admin", "System"
goal TEXT, -- "Create verified account"
success_criteria TEXT, -- JSON array of criteria
order_index INTEGER, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (user_story_id) REFERENCES user_storys(id) ON DELETE CASCADE )

TABLE user_storys ( id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))), module_id TEXT NOT NULL, name TEXT NOT NULL, description TEXT, order_index INTEGER, created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE )

Gua pakai Database Sqlite, file nya ada di sini :/home/sfcore/server-db/SFCoreProTM.db
Untuk API nya, cover Endpoint UNTUK proses SELECT/GET, DELETE/Remove aja dulu, untuk Endpoint INSERT DAN UPDATE NYA NANTI DULU, soal nya beda case.

Setelah itu buatkan proyek frontend nya pakai stack ini :
React - fast, simple, banyak library
Zustand - state management (lebih ringan dari Redux)
React Flow - buat visual graph/dependency
TailwindCSS - styling cepet

nama folder project nya "sfcore-protm-app" di dalam folder sfcore-client-app.

UI client app :

* Ada Header, di Header ada Menu dengan Project App , jadi ketika di klik akan keluar Mega menu di bawah nya berisi semua link halaman berdasarkan Table diatas.
* Untuk setiap halaman , jangan dulu di buatkan, soal nyabelum fix. Fokus ke halaman awal saja yang akan di jelaskan di bawah ini.
* Gak perlu form login, aplikasi langsung terbuka dengan tampilan sebelah kiri seperti sidebar menu tetapi itu load data project.di sebelah kanan adalah body main nya. jadi nanti kalau di klik nama project nya, akan muncul list-list module bentuk tree-view gitu bro, kayak struktur folder tetapi betuk nya kotak ya bro kalau bisa :

    -------------------------------------------------------
  |-| Module A                      button add-edit-delete|
  | -------------------------------------------------------
  |   |
  |   |   -------------------------------------------------------
  |   |---| User Story A                  button add-edit-delete|
  |   |   |------------------------------------------------------
  |   |
  |   |   -------------------------------------------------------
  |   |---| User Story B                  button add-edit-delete|
  |   |   |------------------------------------------------------
  |         |
  |         |   |-----------------------------------------------------|
  |         |---| Use Case ABC                  button add-edit-delete|
  |         |   |-----------------------------------------------------|
  |
  |
  |
  | -------------------------------------------------------
  |-| Module A                      button add-edit-delete|
  | -------------------------------------------------------
  
Jangan pakai Canvas bro kalau bisa.....dan itu akan hirarki terus :
1 Project --> Punya banyak Module --> Punya banyak User Story --> Punya banyak Use Case By Persona --> Punya banyak Task --> Punya banyak Flow Of Task --> Punya banyak Dependency
