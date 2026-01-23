// ============================================================================
// React Query Hooks
// File: src/hooks/useQueries.ts
// ============================================================================

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { userService, CreateUserRequest, UpdateUserRequest } from '@/services/user.service';
import { groupService, CreateGroupRequest, UpdateGroupRequest } from '@/services/group.service';
import { menuService, CreateMenuRequest, UpdateMenuRequest } from '@/services/menu.service';
import { tenantService, CreateTenantRequest, UpdateTenantRequest } from '@/services/tenant.service';
import { clientAppService, CreateClientAppRequest, UpdateClientAppRequest } from '@/services/client-app.service';

// ============================================================================
// User Hooks
// ============================================================================

export const useUsers = (params?: { page?: number; per_page?: number; search?: string }) =>
  useQuery({
    queryKey: ['users', params],
    queryFn: () => userService.getAll(params),
  });

export const useUser = (id: string) =>
  useQuery({
    queryKey: ['users', id],
    queryFn: () => userService.getById(id),
    enabled: !!id,
  });

export const useCreateUser = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateUserRequest) => userService.create(data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['users'] }),
  });
};

export const useUpdateUser = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateUserRequest }) => userService.update(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['users'] }),
  });
};

export const useDeleteUser = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => userService.delete(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['users'] }),
  });
};

// ============================================================================
// Group Hooks
// ============================================================================

export const useGroups = (params?: { page?: number; per_page?: number; search?: string }) =>
  useQuery({
    queryKey: ['groups', params],
    queryFn: () => groupService.getAll(params),
  });

export const useCreateGroup = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateGroupRequest) => groupService.create(data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['groups'] }),
  });
};

export const useUpdateGroup = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateGroupRequest }) => groupService.update(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['groups'] }),
  });
};

export const useDeleteGroup = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => groupService.delete(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['groups'] }),
  });
};

// ============================================================================
// Menu Hooks
// ============================================================================

export const useMenus = (clientAppId?: string) =>
  useQuery({
    queryKey: ['menus', clientAppId],
    queryFn: () => menuService.getTree(clientAppId),
  });

export const useCreateMenu = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateMenuRequest) => menuService.create(data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['menus'] }),
  });
};

export const useUpdateMenu = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateMenuRequest }) => menuService.update(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['menus'] }),
  });
};

export const useDeleteMenu = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => menuService.delete(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['menus'] }),
  });
};

// ============================================================================
// Tenant Hooks
// ============================================================================

export const useTenants = (params?: { page?: number; per_page?: number; search?: string }) =>
  useQuery({
    queryKey: ['tenants', params],
    queryFn: () => tenantService.getAll(params),
  });

export const useCreateTenant = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateTenantRequest) => tenantService.create(data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['tenants'] }),
  });
};

export const useUpdateTenant = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateTenantRequest }) => tenantService.update(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['tenants'] }),
  });
};

export const useDeleteTenant = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => tenantService.delete(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['tenants'] }),
  });
};

// ============================================================================
// Client App Hooks
// ============================================================================

export const useClientApps = (params?: { page?: number; per_page?: number; search?: string }) =>
  useQuery({
    queryKey: ['client-apps', params],
    queryFn: () => clientAppService.getAll(params),
  });

export const useCreateClientApp = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateClientAppRequest) => clientAppService.create(data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['client-apps'] }),
  });
};

export const useUpdateClientApp = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateClientAppRequest }) => clientAppService.update(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['client-apps'] }),
  });
};

export const useDeleteClientApp = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => clientAppService.delete(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['client-apps'] }),
  });
};

export const useRegenerateClientSecret = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => clientAppService.regenerateSecret(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['client-apps'] }),
  });
};
