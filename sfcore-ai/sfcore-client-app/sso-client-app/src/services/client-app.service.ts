// ============================================================================
// Client App Service
// File: src/services/client-app.service.ts
// ============================================================================

import { api, ApiResponse, PaginatedResponse } from './api';

export interface ClientApp {
  id: string;
  app_name: string;
  client_id: string;
  client_secret: string;
  redirect_uris: string[];
  is_active: boolean;
  app_type: 'web' | 'mobile' | 'spa' | 'machine';
  created_at: string;
}

export interface CreateClientAppRequest {
  app_name: string;
  app_type: 'web' | 'mobile' | 'spa' | 'machine';
  redirect_uris?: string[];
  is_active?: boolean;
}

export interface UpdateClientAppRequest {
  app_name?: string;
  redirect_uris?: string[];
  is_active?: boolean;
}

export const clientAppService = {
  async getAll(params?: { page?: number; per_page?: number; search?: string }): Promise<PaginatedResponse<ClientApp>> {
    const response = await api.get<ApiResponse<PaginatedResponse<ClientApp>>>('/client-apps', { params });
    return response.data.data;
  },

  async getById(id: string): Promise<ClientApp> {
    const response = await api.get<ApiResponse<ClientApp>>(`/client-apps/${id}`);
    return response.data.data;
  },

  async create(data: CreateClientAppRequest): Promise<ClientApp> {
    const response = await api.post<ApiResponse<ClientApp>>('/client-apps', data);
    return response.data.data;
  },

  async update(id: string, data: UpdateClientAppRequest): Promise<ClientApp> {
    const response = await api.put<ApiResponse<ClientApp>>(`/client-apps/${id}`, data);
    return response.data.data;
  },

  async delete(id: string): Promise<void> {
    await api.delete(`/client-apps/${id}`);
  },

  async regenerateSecret(id: string): Promise<{ client_secret: string }> {
    const response = await api.post<ApiResponse<{ client_secret: string }>>(`/client-apps/${id}/regenerate-secret`);
    return response.data.data;
  },
};
