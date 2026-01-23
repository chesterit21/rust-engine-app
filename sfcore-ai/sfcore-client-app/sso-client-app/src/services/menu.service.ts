// ============================================================================
// Menu Service
// File: src/services/menu.service.ts
// ============================================================================

import { api, ApiResponse } from './api';

export interface MenuItem {
  id: string;
  menu_name: string;
  menu_url: string;
  menu_icon: string;
  menu_order: number;
  level: number;
  parent_id: string | null;
  is_active: boolean;
  client_app_id: string;
  created_at: string;
  children?: MenuItem[];
}

export interface CreateMenuRequest {
  menu_name: string;
  menu_url: string;
  menu_icon?: string;
  menu_order?: number;
  parent_id?: string | null;
  is_active?: boolean;
  client_app_id: string;
}

export interface UpdateMenuRequest {
  menu_name?: string;
  menu_url?: string;
  menu_icon?: string;
  menu_order?: number;
  parent_id?: string | null;
  is_active?: boolean;
}

export const menuService = {
  async getAll(clientAppId?: string): Promise<MenuItem[]> {
    const params = clientAppId ? { client_app_id: clientAppId } : {};
    const response = await api.get<ApiResponse<MenuItem[]>>('/menus', { params });
    return response.data.data;
  },

  async getTree(clientAppId?: string): Promise<MenuItem[]> {
    const params = clientAppId ? { client_app_id: clientAppId } : {};
    const response = await api.get<ApiResponse<MenuItem[]>>('/menus/tree', { params });
    return response.data.data;
  },

  async getById(id: string): Promise<MenuItem> {
    const response = await api.get<ApiResponse<MenuItem>>(`/menus/${id}`);
    return response.data.data;
  },

  async create(data: CreateMenuRequest): Promise<MenuItem> {
    const response = await api.post<ApiResponse<MenuItem>>('/menus', data);
    return response.data.data;
  },

  async update(id: string, data: UpdateMenuRequest): Promise<MenuItem> {
    const response = await api.put<ApiResponse<MenuItem>>(`/menus/${id}`, data);
    return response.data.data;
  },

  async delete(id: string): Promise<void> {
    await api.delete(`/menus/${id}`);
  },

  async reorder(menuId: string, newOrder: number, newParentId?: string | null): Promise<void> {
    await api.post(`/menus/${menuId}/reorder`, { 
      menu_order: newOrder, 
      parent_id: newParentId 
    });
  },
};
