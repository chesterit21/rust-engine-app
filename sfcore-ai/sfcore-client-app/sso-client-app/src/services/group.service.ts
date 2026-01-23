// ============================================================================
// Group Service
// File: src/services/group.service.ts
// ============================================================================

import { api, ApiResponse, PaginatedResponse } from './api';

export interface Group {
  id: string;
  group_name: string;
  group_description: string;
  is_active: boolean;
  member_count: number;
  created_at: string;
  updated_at: string;
}

export interface CreateGroupRequest {
  group_name: string;
  group_description?: string;
  is_active?: boolean;
}

export interface UpdateGroupRequest {
  group_name?: string;
  group_description?: string;
  is_active?: boolean;
}

export const groupService = {
  async getAll(params?: { page?: number; per_page?: number; search?: string }): Promise<PaginatedResponse<Group>> {
    const response = await api.get<ApiResponse<PaginatedResponse<Group>>>('/groups', { params });
    return response.data.data;
  },

  async getById(id: string): Promise<Group> {
    const response = await api.get<ApiResponse<Group>>(`/groups/${id}`);
    return response.data.data;
  },

  async create(data: CreateGroupRequest): Promise<Group> {
    const response = await api.post<ApiResponse<Group>>('/groups', data);
    return response.data.data;
  },

  async update(id: string, data: UpdateGroupRequest): Promise<Group> {
    const response = await api.put<ApiResponse<Group>>(`/groups/${id}`, data);
    return response.data.data;
  },

  async delete(id: string): Promise<void> {
    await api.delete(`/groups/${id}`);
  },

  async addMember(groupId: string, userId: string): Promise<void> {
    await api.post(`/groups/${groupId}/members`, { user_id: userId });
  },

  async removeMember(groupId: string, userId: string): Promise<void> {
    await api.delete(`/groups/${groupId}/members/${userId}`);
  },

  async getMembers(groupId: string): Promise<{ id: string; display_name: string; email: string }[]> {
    const response = await api.get<ApiResponse<{ id: string; display_name: string; email: string }[]>>(
      `/groups/${groupId}/members`
    );
    return response.data.data;
  },
};
