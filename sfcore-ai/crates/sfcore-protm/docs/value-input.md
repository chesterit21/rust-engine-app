[
 {
    "id": "rblf1a2b3c4d5e6f", "parent_id" : "NONE", "stack_id" : "b300ad08-762b-4c72-a49a-a219e58c8158", "name" : "src", "type" : "FE", "version" : "LITE", "layer_rules" : "Root directory for source code. Architecture Pattern: Component-Based (Standard React) optimized for LITE scale. Separates UI logic, Business logic, and Assets. Focus on rapid prototyping using Bootstrap components. Reference: React Official Documentation.", "order_index" : "1", "naming_conventions" : "lowercase (src)"
 },
 {
    "id": "rblf2b3c4d5e6f7g", "parent_id" : "rblf1a2b3c4d5e6f", "stack_id" : "b300ad08-762b-4c72-a49a-a219e58c8158", "name" : "components", "type" : "FE", "version" : "LITE", "layer_rules" : "Contains reusable UI components (Presentational/Dumb components). These components rely on Bootstrap 5.3+ for styling and accept data via props. No business logic or API calls should reside here.", "order_index" : "2", "naming_conventions" : "PascalCase for files (e.g., Button.js, Card.js), kebab-case for folders if nested."
 },
 {
    "id": "rblf3c4d5e6f7g8h", "parent_id" : "rblf1a2b3c4d5e6f", "stack_id" : "b300ad08-762b-4c72-a49a-a219e58c8158", "name" : "pages", "type" : "FE", "version" : "LITE", "layer_rules" : "Contains Page-level components (Smart/Container components). These correspond to application routes. They assemble Components, handle local state logic via hooks, and fetch data.", "order_index" : "3", "naming_conventions" : "PascalCase for files (e.g., HomePage.js, LoginPage.js)."
 },
 {
    "id": "rblf4d5e6f7g8h9i", "parent_id" : "rblf1a2b3c4d5e6f", "stack_id" : "b300ad08-762b-4c72-a49a-a219e58c8158", "name" : "hooks", "type" : "FE", "version" : "LITE", "layer_rules" : "Contains reusable Custom Hooks. Encapsulates side effects, form logic, or data fetching logic to keep components clean. Shared logic across pages goes here.", "order_index" : "4", "naming_conventions" : "camelCase with 'use' prefix (e.g., useAuth.js, useFetch.js)."
 },
 {
    "id": "rblf5e6f7g8h9i0j", "parent_id" : "rblf1a2b3c4d5e6f", "stack_id" : "b300ad08-762b-4c72-a49a-a219e58c8158", "name" : "services", "type" : "FE", "version" : "LITE", "layer_rules" : "Handles external communication (API calls). Functions to interact with backends or third-party services. Ensures separation between UI and data fetching logic.", "order_index" : "5", "naming_conventions" : "camelCase (e.g., apiService.js, userService.js)."
 },
 {
    "id": "rblf6f7g8h9i0j1k", "parent_id" : "rblf1a2b3c4d5e6f", "stack_id" : "b300ad08-762b-4c72-a49a-a219e58c8158", "name" : "context", "type" : "FE", "version" : "LITE", "layer_rules" : "Contains React Context providers for global state management (e.g., User Theme, Auth State). Suitable for LITE scale instead of complex state libraries like Redux.", "order_index" : "6", "naming_conventions" : "PascalCase with 'Context' suffix (e.g., AuthContext.js)."
 },
 {
    "id": "rblf7g8h9i0j1k2l", "parent_id" : "rblf1a2b3c4d5e6f", "stack_id" : "b300ad08-762b-4c72-a49a-a219e58c8158", "name" : "utils", "type" : "FE", "version" : "LITE", "layer_rules" : "Contains helper functions, constants, and pure logic utilities (e.g., date formatters, validators). No dependencies on React component lifecycle.", "order_index" : "7", "naming_conventions" : "camelCase (e.g., formatDate.js, constants.js)."
 },
 {
    "id": "rblf8h9i0j1k2l3m", "parent_id" : "rblf1a2b3c4d5e6f", "stack_id" : "b300ad08-762b-4c72-a49a-a219e58c8158", "name" : "assets", "type" : "FE", "version" : "LITE", "layer_rules" : "Static assets such as images, fonts, icons, and global CSS/SCSS files for Bootstrap customization.", "order_index" : "8", "naming_conventions" : "kebab-case (e.g., logo-image.png, custom-styles.scss)."
 }
]
