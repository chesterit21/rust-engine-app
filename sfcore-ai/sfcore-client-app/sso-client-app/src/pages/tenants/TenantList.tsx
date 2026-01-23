// ============================================================================
// Tenant List Page with Full CRUD
// File: src/pages/tenants/TenantList.tsx
// ============================================================================

import React, { useState, useMemo } from 'react';
import { ColumnDef } from '@tanstack/react-table';
import { Plus, Edit, Trash2, Building2, Globe, Users, CheckCircle, XCircle } from 'lucide-react';
import { Table } from '@/components/ui/Table';
import { Button } from '@/components/ui/Button';
import { Modal } from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';

// ============================================================================
// Types
// ============================================================================

interface Tenant {
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

// ============================================================================
// Mock Data
// ============================================================================

const generateMockTenants = (): Tenant[] => {
  const tenants = [
    { name: 'Acme Corporation', code: 'ACME', domain: 'acme.example.com', max: 100, current: 45 },
    { name: 'TechStart Inc', code: 'TECH', domain: 'techstart.io', max: 50, current: 32 },
    { name: 'Global Solutions', code: 'GLOB', domain: 'globalsol.com', max: 200, current: 156 },
    { name: 'InnovateCo', code: 'INNO', domain: 'innovateco.net', max: 75, current: 28 },
    { name: 'DataDriven Ltd', code: 'DATA', domain: 'datadriven.co', max: 150, current: 89 },
    { name: 'CloudFirst', code: 'CLOD', domain: 'cloudfirst.app', max: 100, current: 67 },
    { name: 'NextGen Systems', code: 'NEXT', domain: 'nextgen.sys', max: 80, current: 12 },
    { name: 'Enterprise Plus', code: 'ENTP', domain: 'enterprise-plus.com', max: 500, current: 423 },
  ];

  return tenants.map((t, i) => ({
    id: `tenant-${String(i + 1).padStart(3, '0')}`,
    tenant_name: t.name,
    tenant_code: t.code,
    domain: t.domain,
    is_active: i !== 6,
    max_users: t.max,
    current_users: t.current,
    created_at: new Date(Date.now() - i * 86400000 * 30).toISOString(),
    updated_at: new Date(Date.now() - i * 86400000 * 5).toISOString(),
  }));
};

// ============================================================================
// Tenant Form
// ============================================================================

interface TenantFormProps {
  tenant?: Tenant | null;
  onSubmit: (data: Partial<Tenant>) => void;
  onCancel: () => void;
}

const TenantForm: React.FC<TenantFormProps> = ({ tenant, onSubmit, onCancel }) => {
  const [formData, setFormData] = useState({
    tenant_name: tenant?.tenant_name || '',
    tenant_code: tenant?.tenant_code || '',
    domain: tenant?.domain || '',
    max_users: tenant?.max_users || 50,
    is_active: tenant?.is_active ?? true,
  });
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = () => {
    const e: Record<string, string> = {};
    if (!formData.tenant_name.trim()) e.tenant_name = 'Name required';
    if (!formData.tenant_code.trim()) e.tenant_code = 'Code required';
    if (formData.tenant_code.length > 10) e.tenant_code = 'Max 10 characters';
    if (!formData.domain.trim()) e.domain = 'Domain required';
    setErrors(e);
    return Object.keys(e).length === 0;
  };

  const handleSubmit = (ev: React.FormEvent) => {
    ev.preventDefault();
    if (validate()) onSubmit(formData);
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <Input
        label="Tenant Name"
        placeholder="Company name"
        value={formData.tenant_name}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, tenant_name: e.target.value })}
        error={errors.tenant_name}
      />
      <div className="grid grid-cols-2 gap-4">
        <Input
          label="Tenant Code"
          placeholder="ACME"
          value={formData.tenant_code}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, tenant_code: e.target.value.toUpperCase() })}
          error={errors.tenant_code}
        />
        <Input
          label="Max Users"
          type="number"
          value={formData.max_users.toString()}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, max_users: parseInt(e.target.value) || 50 })}
        />
      </div>
      <Input
        label="Domain"
        placeholder="company.example.com"
        value={formData.domain}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, domain: e.target.value })}
        error={errors.domain}
        icon={<Globe className="w-4 h-4" />}
      />
      <div className="flex items-center gap-3">
        <input type="checkbox" id="is_active" checked={formData.is_active}
          onChange={(e) => setFormData({ ...formData, is_active: e.target.checked })}
          className="w-4 h-4 text-primary-600 rounded" />
        <label htmlFor="is_active" className="text-sm text-gray-700 dark:text-gray-300">Tenant is active</label>
      </div>
      <div className="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-dark-lighter">
        <Button type="button" variant="secondary" onClick={onCancel}>Cancel</Button>
        <Button type="submit">{tenant ? 'Update' : 'Create'} Tenant</Button>
      </div>
    </form>
  );
};

// ============================================================================
// Main Component
// ============================================================================

export const TenantList: React.FC = () => {
  const [tenants, setTenants] = useState<Tenant[]>(generateMockTenants);
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [isDeleteOpen, setIsDeleteOpen] = useState(false);
  const [selected, setSelected] = useState<Tenant | null>(null);

  const columns = useMemo<ColumnDef<Tenant>[]>(() => [
    {
      accessorKey: 'tenant_name',
      header: 'Tenant',
      cell: ({ row }) => (
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-gradient-to-br from-emerald-400 to-emerald-600 rounded-xl flex items-center justify-center">
            <Building2 className="w-5 h-5 text-white" />
          </div>
          <div>
            <div className="font-medium text-gray-900 dark:text-white">{row.original.tenant_name}</div>
            <div className="text-sm text-gray-500">{row.original.tenant_code}</div>
          </div>
        </div>
      ),
    },
    {
      accessorKey: 'domain',
      header: 'Domain',
      cell: ({ getValue }) => (
        <span className="flex items-center gap-1.5 text-sm">
          <Globe className="w-4 h-4 text-gray-400" />
          {getValue() as string}
        </span>
      ),
    },
    {
      accessorKey: 'current_users',
      header: 'Users',
      cell: ({ row }) => (
        <div className="flex items-center gap-2">
          <Users className="w-4 h-4 text-gray-400" />
          <span>{row.original.current_users} / {row.original.max_users}</span>
          <div className="w-16 h-2 bg-gray-200 dark:bg-dark-lighter rounded-full overflow-hidden">
            <div 
              className="h-full bg-primary-500 rounded-full"
              style={{ width: `${(row.original.current_users / row.original.max_users) * 100}%` }}
            />
          </div>
        </div>
      ),
    },
    {
      accessorKey: 'is_active',
      header: 'Status',
      cell: ({ getValue }) => getValue() ? (
        <span className="flex items-center gap-1 text-green-600"><CheckCircle className="w-4 h-4" /> Active</span>
      ) : (
        <span className="flex items-center gap-1 text-gray-400"><XCircle className="w-4 h-4" /> Inactive</span>
      ),
    },
    {
      id: 'actions',
      header: 'Actions',
      enableSorting: false,
      cell: ({ row }) => (
        <div className="flex gap-1">
          <button onClick={() => { setSelected(row.original); setIsFormOpen(true); }}
            className="p-1.5 hover:bg-gray-100 dark:hover:bg-dark-lighter rounded-lg"><Edit className="w-4 h-4 text-gray-500" /></button>
          <button onClick={() => { setSelected(row.original); setIsDeleteOpen(true); }}
            className="p-1.5 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg"><Trash2 className="w-4 h-4 text-red-500" /></button>
        </div>
      ),
    },
  ], []);

  const handleSubmit = (data: Partial<Tenant>) => {
    if (selected) {
      setTenants(prev => prev.map(t => t.id === selected.id ? { ...t, ...data } : t));
    } else {
      setTenants(prev => [{ id: `tenant-${Date.now()}`, ...data, current_users: 0, created_at: new Date().toISOString(), updated_at: new Date().toISOString() } as Tenant, ...prev]);
    }
    setIsFormOpen(false); setSelected(null);
  };

  const handleDelete = () => {
    if (selected) setTenants(prev => prev.filter(t => t.id !== selected.id));
    setIsDeleteOpen(false); setSelected(null);
  };

  return (
    <div className="space-y-6 animate-fade-in">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white font-display">Tenant Management</h1>
          <p className="mt-1 text-gray-500 dark:text-gray-400">Manage multi-tenant organizations ({tenants.length} total)</p>
        </div>
        <Button onClick={() => { setSelected(null); setIsFormOpen(true); }}><Plus className="w-4 h-4" /> Add Tenant</Button>
      </div>

      <Table data={tenants} columns={columns} searchable exportable pageSize={10} />

      <Modal isOpen={isFormOpen} onClose={() => { setIsFormOpen(false); setSelected(null); }} title={selected ? 'Edit Tenant' : 'Create Tenant'} size="md" closeOnClickOutside={false}>
        <TenantForm tenant={selected} onSubmit={handleSubmit} onCancel={() => { setIsFormOpen(false); setSelected(null); }} />
      </Modal>

      <Modal isOpen={isDeleteOpen} onClose={() => { setIsDeleteOpen(false); setSelected(null); }} title="Confirm Delete" size="sm" closeOnClickOutside={false}>
        {selected && (
          <div className="text-center space-y-4">
            <div className="w-16 h-16 mx-auto bg-red-100 rounded-full flex items-center justify-center"><Trash2 className="w-8 h-8 text-red-600" /></div>
            <p className="text-gray-500">Delete <strong>{selected.tenant_name}</strong>?</p>
            <div className="flex justify-center gap-3">
              <Button variant="secondary" onClick={() => { setIsDeleteOpen(false); setSelected(null); }}>Cancel</Button>
              <Button variant="danger" onClick={handleDelete}>Delete</Button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
};
