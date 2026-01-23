// ============================================================================
// Client App List Page with Full CRUD
// File: src/pages/client-apps/ClientAppList.tsx
// ============================================================================

import React, { useState, useMemo } from 'react';
import { ColumnDef } from '@tanstack/react-table';
import { Plus, Edit, Trash2, Box, Key, Copy, CheckCircle, XCircle, RefreshCw } from 'lucide-react';
import { Table } from '@/components/ui/Table';
import { Button } from '@/components/ui/Button';
import { Modal } from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';

// ============================================================================
// Types
// ============================================================================

interface ClientApp {
  id: string;
  app_name: string;
  client_id: string;
  client_secret: string;
  redirect_uris: string[];
  is_active: boolean;
  app_type: 'web' | 'mobile' | 'spa' | 'machine';
  created_at: string;
}

// ============================================================================
// Mock Data
// ============================================================================

const generateMockApps = (): ClientApp[] => {
  const apps = [
    { name: 'Admin Dashboard', type: 'spa' as const, uris: ['http://localhost:5173/callback', 'https://admin.example.com/callback'] },
    { name: 'Mobile App iOS', type: 'mobile' as const, uris: ['myapp://callback'] },
    { name: 'Mobile App Android', type: 'mobile' as const, uris: ['myapp://callback'] },
    { name: 'Customer Portal', type: 'web' as const, uris: ['https://portal.example.com/auth/callback'] },
    { name: 'API Service', type: 'machine' as const, uris: [] },
    { name: 'Partner Integration', type: 'web' as const, uris: ['https://partner.external.com/oauth2/callback'] },
  ];

  return apps.map((a, i) => ({
    id: `app-${String(i + 1).padStart(3, '0')}`,
    app_name: a.name,
    client_id: `client_${Math.random().toString(36).slice(2, 14)}`,
    client_secret: `secret_${Math.random().toString(36).slice(2, 26)}`,
    redirect_uris: a.uris,
    is_active: i !== 4,
    app_type: a.type,
    created_at: new Date(Date.now() - i * 86400000 * 15).toISOString(),
  }));
};

// ============================================================================
// Client App Form
// ============================================================================

interface AppFormProps {
  app?: ClientApp | null;
  onSubmit: (data: Partial<ClientApp>) => void;
  onCancel: () => void;
}

const AppForm: React.FC<AppFormProps> = ({ app, onSubmit, onCancel }) => {
  const [formData, setFormData] = useState({
    app_name: app?.app_name || '',
    app_type: app?.app_type || 'web',
    redirect_uris: app?.redirect_uris.join('\n') || '',
    is_active: app?.is_active ?? true,
  });
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = () => {
    const e: Record<string, string> = {};
    if (!formData.app_name.trim()) e.app_name = 'App name required';
    if (formData.app_type !== 'machine' && !formData.redirect_uris.trim()) e.redirect_uris = 'At least one redirect URI required';
    setErrors(e);
    return Object.keys(e).length === 0;
  };

  const handleSubmit = (ev: React.FormEvent) => {
    ev.preventDefault();
    if (validate()) {
      onSubmit({
        app_name: formData.app_name,
        app_type: formData.app_type as ClientApp['app_type'],
        redirect_uris: formData.redirect_uris.split('\n').filter(u => u.trim()),
        is_active: formData.is_active,
      });
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <Input
        label="App Name"
        placeholder="My Application"
        value={formData.app_name}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, app_name: e.target.value })}
        error={errors.app_name}
      />
      <div className="space-y-1">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">App Type</label>
        <select value={formData.app_type} onChange={(e) => setFormData({ ...formData, app_type: e.target.value as ClientApp['app_type'] })}
          className="w-full px-4 py-2.5 border border-gray-300 rounded-lg dark:bg-dark-lighter dark:border-dark-lighter dark:text-white">
          <option value="web">Web Application</option>
          <option value="spa">Single Page App (SPA)</option>
          <option value="mobile">Mobile App</option>
          <option value="machine">Machine-to-Machine</option>
        </select>
      </div>
      {formData.app_type !== 'machine' && (
        <div className="space-y-1">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">Redirect URIs (one per line)</label>
          <textarea value={formData.redirect_uris} onChange={(e) => setFormData({ ...formData, redirect_uris: e.target.value })}
            rows={3} placeholder="https://yourapp.com/callback" 
            className="w-full px-4 py-2.5 border border-gray-300 rounded-lg dark:bg-dark-lighter dark:border-dark-lighter dark:text-white resize-none" />
          {errors.redirect_uris && <p className="text-sm text-red-500">{errors.redirect_uris}</p>}
        </div>
      )}
      <div className="flex items-center gap-3">
        <input type="checkbox" id="is_active" checked={formData.is_active}
          onChange={(e) => setFormData({ ...formData, is_active: e.target.checked })} className="w-4 h-4 text-primary-600 rounded" />
        <label htmlFor="is_active" className="text-sm text-gray-700 dark:text-gray-300">App is active</label>
      </div>
      <div className="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-dark-lighter">
        <Button type="button" variant="secondary" onClick={onCancel}>Cancel</Button>
        <Button type="submit">{app ? 'Update' : 'Create'} App</Button>
      </div>
    </form>
  );
};

// ============================================================================
// Main Component
// ============================================================================

export const ClientAppList: React.FC = () => {
  const [apps, setApps] = useState<ClientApp[]>(generateMockApps);
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [isDeleteOpen, setIsDeleteOpen] = useState(false);
  const [isSecretOpen, setIsSecretOpen] = useState(false);
  const [selected, setSelected] = useState<ClientApp | null>(null);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    // Could add toast notification here
  };

  const typeColors = {
    web: 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400',
    spa: 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400',
    mobile: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
    machine: 'bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400',
  };

  const columns = useMemo<ColumnDef<ClientApp>[]>(() => [
    {
      accessorKey: 'app_name',
      header: 'Application',
      cell: ({ row }) => (
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-gradient-to-br from-violet-400 to-violet-600 rounded-xl flex items-center justify-center">
            <Box className="w-5 h-5 text-white" />
          </div>
          <div>
            <div className="font-medium text-gray-900 dark:text-white">{row.original.app_name}</div>
            <div className="text-xs text-gray-500 font-mono">{row.original.client_id.slice(0, 20)}...</div>
          </div>
        </div>
      ),
    },
    {
      accessorKey: 'app_type',
      header: 'Type',
      cell: ({ getValue }) => (
        <span className={`px-2.5 py-1 rounded-full text-xs font-medium ${typeColors[getValue() as keyof typeof typeColors]}`}>
          {(getValue() as string).toUpperCase()}
        </span>
      ),
    },
    {
      accessorKey: 'redirect_uris',
      header: 'Redirects',
      cell: ({ getValue }) => {
        const uris = getValue() as string[];
        return <span className="text-sm text-gray-500">{uris.length} URI{uris.length !== 1 ? 's' : ''}</span>;
      },
    },
    {
      accessorKey: 'is_active',
      header: 'Status',
      cell: ({ getValue }) => getValue() ? (
        <span className="flex items-center gap-1 text-green-600"><CheckCircle className="w-4 h-4" /> Active</span>
      ) : (
        <span className="flex items-center gap-1 text-gray-400"><XCircle className="w-4 h-4" /> Disabled</span>
      ),
    },
    {
      id: 'actions',
      header: 'Actions',
      enableSorting: false,
      cell: ({ row }) => (
        <div className="flex gap-1">
          <button onClick={() => { setSelected(row.original); setIsSecretOpen(true); }} title="View credentials"
            className="p-1.5 hover:bg-gray-100 dark:hover:bg-dark-lighter rounded-lg"><Key className="w-4 h-4 text-gray-500" /></button>
          <button onClick={() => { setSelected(row.original); setIsFormOpen(true); }} title="Edit"
            className="p-1.5 hover:bg-gray-100 dark:hover:bg-dark-lighter rounded-lg"><Edit className="w-4 h-4 text-gray-500" /></button>
          <button onClick={() => { setSelected(row.original); setIsDeleteOpen(true); }} title="Delete"
            className="p-1.5 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg"><Trash2 className="w-4 h-4 text-red-500" /></button>
        </div>
      ),
    },
  ], []);

  const handleSubmit = (data: Partial<ClientApp>) => {
    if (selected) {
      setApps(prev => prev.map(a => a.id === selected.id ? { ...a, ...data } : a));
    } else {
      const newApp: ClientApp = {
        id: `app-${Date.now()}`,
        app_name: data.app_name || '',
        client_id: `client_${Math.random().toString(36).slice(2, 14)}`,
        client_secret: `secret_${Math.random().toString(36).slice(2, 26)}`,
        redirect_uris: data.redirect_uris || [],
        is_active: data.is_active ?? true,
        app_type: data.app_type || 'web',
        created_at: new Date().toISOString(),
      };
      setApps(prev => [newApp, ...prev]);
    }
    setIsFormOpen(false); setSelected(null);
  };

  const handleDelete = () => {
    if (selected) setApps(prev => prev.filter(a => a.id !== selected.id));
    setIsDeleteOpen(false); setSelected(null);
  };

  const regenerateSecret = () => {
    if (selected) {
      const newSecret = `secret_${Math.random().toString(36).slice(2, 26)}`;
      setApps(prev => prev.map(a => a.id === selected.id ? { ...a, client_secret: newSecret } : a));
      setSelected({ ...selected, client_secret: newSecret });
    }
  };

  return (
    <div className="space-y-6 animate-fade-in">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white font-display">Client Applications</h1>
          <p className="mt-1 text-gray-500 dark:text-gray-400">Manage OAuth client apps ({apps.length} total)</p>
        </div>
        <Button onClick={() => { setSelected(null); setIsFormOpen(true); }}><Plus className="w-4 h-4" /> Add App</Button>
      </div>

      <Table data={apps} columns={columns} searchable exportable pageSize={10} />

      {/* Form Modal */}
      <Modal isOpen={isFormOpen} onClose={() => { setIsFormOpen(false); setSelected(null); }} title={selected ? 'Edit App' : 'Create App'} size="md" closeOnClickOutside={false}>
        <AppForm app={selected} onSubmit={handleSubmit} onCancel={() => { setIsFormOpen(false); setSelected(null); }} />
      </Modal>

      {/* Credentials Modal */}
      <Modal isOpen={isSecretOpen} onClose={() => { setIsSecretOpen(false); setSelected(null); }} title="Client Credentials" size="md" closeOnClickOutside={false}>
        {selected && (
          <div className="space-y-4">
            <div className="p-4 bg-gray-50 dark:bg-dark-lighter rounded-lg space-y-3">
              <div>
                <label className="text-xs font-medium text-gray-500 uppercase">Client ID</label>
                <div className="flex items-center gap-2 mt-1">
                  <code className="flex-1 px-3 py-2 bg-white dark:bg-dark-light border rounded font-mono text-sm">{selected.client_id}</code>
                  <button onClick={() => copyToClipboard(selected.client_id)} className="p-2 hover:bg-gray-200 dark:hover:bg-dark-light rounded"><Copy className="w-4 h-4" /></button>
                </div>
              </div>
              <div>
                <label className="text-xs font-medium text-gray-500 uppercase">Client Secret</label>
                <div className="flex items-center gap-2 mt-1">
                  <code className="flex-1 px-3 py-2 bg-white dark:bg-dark-light border rounded font-mono text-sm">{selected.client_secret}</code>
                  <button onClick={() => copyToClipboard(selected.client_secret)} className="p-2 hover:bg-gray-200 dark:hover:bg-dark-light rounded"><Copy className="w-4 h-4" /></button>
                </div>
              </div>
            </div>
            <div className="flex justify-between">
              <Button variant="secondary" onClick={regenerateSecret}><RefreshCw className="w-4 h-4" /> Regenerate Secret</Button>
              <Button onClick={() => { setIsSecretOpen(false); setSelected(null); }}>Close</Button>
            </div>
          </div>
        )}
      </Modal>

      {/* Delete Modal */}
      <Modal isOpen={isDeleteOpen} onClose={() => { setIsDeleteOpen(false); setSelected(null); }} title="Confirm Delete" size="sm" closeOnClickOutside={false}>
        {selected && (
          <div className="text-center space-y-4">
            <div className="w-16 h-16 mx-auto bg-red-100 rounded-full flex items-center justify-center"><Trash2 className="w-8 h-8 text-red-600" /></div>
            <p className="text-gray-500">Delete <strong>{selected.app_name}</strong>?<br/><span className="text-red-500 text-sm">This will revoke all access tokens!</span></p>
            <div className="flex justify-center gap-3">
              <Button variant="secondary" onClick={() => { setIsDeleteOpen(false); setSelected(null); }}>Cancel</Button>
              <Button variant="danger" onClick={handleDelete}>Delete App</Button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
};
