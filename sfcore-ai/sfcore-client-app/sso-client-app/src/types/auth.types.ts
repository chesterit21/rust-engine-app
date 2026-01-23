export interface User {
  id: string;
  display_name: string;
  email: string;
  status_member: 'active' | 'wait_validation' | 'suspended' | 'blocked' | 'new_register' | 'invitation';
  is_active: boolean;
  is_login: boolean;
  last_login: string | null;
  created_at: string;
  profile_image?: string;
  email_verified?: boolean;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface LoginRequest {
  email: string;
  password: string;
  remember?: boolean;
}

export interface RegisterData {
  display_name: string;
  email: string;
  password: string;
  confirmPassword?: string;
}

export interface AuthResponse {
  user: User;
  access_token: string;
  refresh_token: string;
}

export interface LoginResponse {
  user: User;
  access_token: string;
  refresh_token: string;
}

export interface UserPermissions {
  can_view_users: boolean;
  can_edit_users: boolean;
  can_delete_users: boolean;
  can_view_roles: boolean;
  can_manage_roles: boolean;
  // Add other permissions as needed based on the backend response
  [key: string]: boolean;
}
