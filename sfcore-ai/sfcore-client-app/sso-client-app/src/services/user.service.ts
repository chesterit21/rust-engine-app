// ============================================================================
// User Service
// File: src/services/user.service.ts
// ============================================================================

import { api, ApiResponse, PaginatedResponse } from './api';

// ============================================================================
// Types
// ============================================================================

export interface User {
  id: string;
  display_name: string;
  email: string;
  status_member: 'active' | 'wait_validation' | 'suspended' | 'blocked';
  is_active: boolean;
  is_login: boolean;
  last_login: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateUserRequest {
  display_name: string;
  email: string;
  password: string;
  status_member?: string;
  is_active?: boolean;
}

export interface UpdateUserRequest {
  display_name?: string;
  email?: string;
  status_member?: string;
  is_active?: boolean;
}

export interface UserQueryParams {
  page?: number;
  per_page?: number;
  search?: string;
  status?: string;
  sort_by?: string;
  sort_order?: 'asc' | 'desc';
}

// ============================================================================
// User Service
// ============================================================================

export const userService = {
  async getAll(params?: UserQueryParams): Promise<PaginatedResponse<User>> {
    const response = await api.get<ApiResponse<PaginatedResponse<User>>>('/users', { params });
    return response.data.data;
  },

  async getById(id: string): Promise<User> {
    const response = await api.get<ApiResponse<User>>(`/users/${id}`);
    return response.data.data;
  },

  async create(data: CreateUserRequest): Promise<User> {
    const response = await api.post<ApiResponse<User>>('/users', data);
    return response.data.data;
  },

  async update(id: string, data: UpdateUserRequest): Promise<User> {
    const response = await api.put<ApiResponse<User>>(`/users/${id}`, data);
    return response.data.data;
  },

  async delete(id: string): Promise<void> {
    await api.delete(`/users/${id}`);
  },

  async activate(id: string): Promise<User> {
    const response = await api.post<ApiResponse<User>>(`/users/${id}/activate`);
    return response.data.data;
  },

  async suspend(id: string, reason?: string): Promise<User> {
    const response = await api.post<ApiResponse<User>>(`/users/${id}/suspend`, { reason });
    return response.data.data;
  },

  async block(id: string, reason?: string): Promise<User> {
    const response = await api.post<ApiResponse<User>>(`/users/${id}/block`, { reason });
    return response.data.data;
  },
};
