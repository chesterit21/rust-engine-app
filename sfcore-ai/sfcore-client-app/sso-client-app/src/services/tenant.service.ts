// ============================================================================
// Tenant Service
// File: src/services/tenant.service.ts
// ============================================================================

import { api, ApiResponse, PaginatedResponse } from './api';

export interface Tenant {
  id: string;
  tenant_name: string;
  tenant_code: string;
  domain: string;
  is_active: boolean;
  max_users: number;
  current_users: number;
  created_at: string;
  updated_at: string;
}

export interface CreateTenantRequest {
  tenant_name: string;
  tenant_code: string;
  domain: string;
  max_users?: number;
  is_active?: boolean;
}

export interface UpdateTenantRequest {
  tenant_name?: string;
  domain?: string;
  max_users?: number;
  is_active?: boolean;
}

export const tenantService = {
  async getAll(params?: { page?: number; per_page?: number; search?: string }): Promise<PaginatedResponse<Tenant>> {
    const response = await api.get<ApiResponse<PaginatedResponse<Tenant>>>('/tenants', { params });
    return response.data.data;
  },

  async getById(id: string): Promise<Tenant> {
    const response = await api.get<ApiResponse<Tenant>>(`/tenants/${id}`);
    return response.data.data;
  },

  async create(data: CreateTenantRequest): Promise<Tenant> {
    const response = await api.post<ApiResponse<Tenant>>('/tenants', data);
    return response.data.data;
  },

  async update(id: string, data: UpdateTenantRequest): Promise<Tenant> {
    const response = await api.put<ApiResponse<Tenant>>(`/tenants/${id}`, data);
    return response.data.data;
  },

  async delete(id: string): Promise<void> {
    await api.delete(`/tenants/${id}`);
  },
};
