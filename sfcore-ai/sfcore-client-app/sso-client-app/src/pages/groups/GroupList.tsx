// ============================================================================
// Group List Page with Full CRUD
// File: src/pages/groups/GroupList.tsx
// Features: TanStack Table, Search, Sort, Pagination, Export, CRUD Modal
// ============================================================================

import React, { useState, useMemo } from 'react';
import { ColumnDef } from '@tanstack/react-table';
import { Plus, Edit, Trash2, Users } from 'lucide-react';
import { Table } from '@/components/ui/Table';
import { Button } from '@/components/ui/Button';
import { Modal } from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';

// ============================================================================
// Types
// ============================================================================

interface Group {
  id: string;
  group_name: string;
  group_description: string;
  is_active: boolean;
  member_count: number;
  created_at: string;
  updated_at: string;
}

// ============================================================================
// Mock Data
// ============================================================================

const generateMockGroups = (): Group[] => {
  const groups = [
    { name: 'Administrators', desc: 'Full system access and control', members: 5 },
    { name: 'Managers', desc: 'Department management capabilities', members: 12 },
    { name: 'Developers', desc: 'Development and testing access', members: 25 },
    { name: 'Support', desc: 'Customer support features', members: 18 },
    { name: 'Finance', desc: 'Financial reports and billing', members: 8 },
    { name: 'HR', desc: 'Human resources management', members: 6 },
    { name: 'Marketing', desc: 'Marketing and analytics tools', members: 10 },
    { name: 'Operations', desc: 'Operations and logistics', members: 15 },
    { name: 'Sales', desc: 'Sales and CRM access', members: 22 },
    { name: 'Guests', desc: 'Limited read-only access', members: 45 },
  ];

  return groups.map((g, i) => ({
    id: `group-${String(i + 1).padStart(3, '0')}`,
    group_name: g.name,
    group_description: g.desc,
    is_active: i !== 9, // Guests inactive
    member_count: g.members,
    created_at: new Date(Date.now() - i * 86400000 * 30).toISOString(),
    updated_at: new Date(Date.now() - i * 86400000 * 5).toISOString(),
  }));
};

// ============================================================================
// Group Form Component
// ============================================================================

interface GroupFormProps {
  group?: Group | null;
  onSubmit: (data: Partial<Group>) => void;
  onCancel: () => void;
}

const GroupForm: React.FC<GroupFormProps> = ({ group, onSubmit, onCancel }) => {
  const [formData, setFormData] = useState({
    group_name: group?.group_name || '',
    group_description: group?.group_description || '',
    is_active: group?.is_active ?? true,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = () => {
    const newErrors: Record<string, string> = {};
    
    if (!formData.group_name.trim()) {
      newErrors.group_name = 'Group name is required';
    } else if (formData.group_name.length < 2) {
      newErrors.group_name = 'Group name must be at least 2 characters';
    }

    if (formData.group_description.length > 200) {
      newErrors.group_description = 'Description must be less than 200 characters';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (validate()) {
      onSubmit(formData);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <Input
        label="Group Name"
        placeholder="Enter group name"
        value={formData.group_name}
        onChange={(e) => setFormData({ ...formData, group_name: e.target.value })}
        error={errors.group_name}
      />

      <div className="space-y-1">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Description
        </label>
        <textarea
          placeholder="Enter group description"
          value={formData.group_description}
          onChange={(e) => setFormData({ ...formData, group_description: e.target.value })}
          rows={3}
          className="w-full px-4 py-2.5 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 dark:bg-dark-lighter dark:border-dark-lighter dark:text-white resize-none"
        />
        {errors.group_description && (
          <p className="text-sm text-red-500">{errors.group_description}</p>
        )}
      </div>

      <div className="flex items-center gap-3">
        <input
          type="checkbox"
          id="is_active"
          checked={formData.is_active}
          onChange={(e) => setFormData({ ...formData, is_active: e.target.checked })}
          className="w-4 h-4 text-primary-600 rounded border-gray-300 focus:ring-primary-500"
        />
        <label htmlFor="is_active" className="text-sm text-gray-700 dark:text-gray-300">
          Group is active
        </label>
      </div>

      <div className="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-dark-lighter">
        <Button type="button" variant="secondary" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit">
          {group ? 'Update Group' : 'Create Group'}
        </Button>
      </div>
    </form>
  );
};

// ============================================================================
// Main GroupList Component
// ============================================================================

export const GroupList: React.FC = () => {
  const [groups, setGroups] = useState<Group[]>(generateMockGroups);
  const [isFormModalOpen, setIsFormModalOpen] = useState(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [selectedGroup, setSelectedGroup] = useState<Group | null>(null);

  // Table columns
  const columns = useMemo<ColumnDef<Group>[]>(() => [
    {
      accessorKey: 'group_name',
      header: 'Group',
      cell: ({ row }) => (
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-gradient-to-br from-purple-400 to-purple-600 rounded-xl flex items-center justify-center">
            <Users className="w-5 h-5 text-white" />
          </div>
          <div>
            <div className="font-medium text-gray-900 dark:text-white">
              {row.original.group_name}
            </div>
            <div className="text-sm text-gray-500 truncate max-w-[250px]">
              {row.original.group_description}
            </div>
          </div>
        </div>
      ),
    },
    {
      accessorKey: 'member_count',
      header: 'Members',
      cell: ({ getValue }) => (
        <span className="flex items-center gap-1.5">
          <Users className="w-4 h-4 text-gray-400" />
          {getValue() as number}
        </span>
      ),
    },
    {
      accessorKey: 'is_active',
      header: 'Status',
      cell: ({ getValue }) => (
        <span className={`px-2.5 py-1 rounded-full text-xs font-medium ${
          getValue() 
            ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
            : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'
        }`}>
          {getValue() ? 'Active' : 'Inactive'}
        </span>
      ),
    },
    {
      accessorKey: 'created_at',
      header: 'Created',
      cell: ({ getValue }) => new Date(getValue() as string).toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
      }),
    },
    {
      id: 'actions',
      header: 'Actions',
      enableSorting: false,
      cell: ({ row }) => (
        <div className="flex items-center gap-2">
          <button
            onClick={(e) => { e.stopPropagation(); handleEdit(row.original); }}
            className="p-1.5 hover:bg-gray-100 dark:hover:bg-dark-lighter rounded-lg transition-colors"
            title="Edit"
          >
            <Edit className="w-4 h-4 text-gray-500" />
          </button>
          <button
            onClick={(e) => { e.stopPropagation(); handleDelete(row.original); }}
            className="p-1.5 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
            title="Delete"
          >
            <Trash2 className="w-4 h-4 text-red-500" />
          </button>
        </div>
      ),
    },
  ], []);

  // Handlers
  const handleCreate = () => { setSelectedGroup(null); setIsFormModalOpen(true); };
  const handleEdit = (group: Group) => { setSelectedGroup(group); setIsFormModalOpen(true); };
  const handleDelete = (group: Group) => { setSelectedGroup(group); setIsDeleteModalOpen(true); };

  const handleFormSubmit = (data: Partial<Group>) => {
    if (selectedGroup) {
      setGroups(prev => prev.map(g => 
        g.id === selectedGroup.id ? { ...g, ...data, updated_at: new Date().toISOString() } : g
      ));
    } else {
      const newGroup: Group = {
        id: `group-${Date.now()}`,
        group_name: data.group_name!,
        group_description: data.group_description || '',
        is_active: data.is_active ?? true,
        member_count: 0,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      setGroups(prev => [newGroup, ...prev]);
    }
    setIsFormModalOpen(false);
    setSelectedGroup(null);
  };

  const handleDeleteConfirm = () => {
    if (selectedGroup) {
      setGroups(prev => prev.filter(g => g.id !== selectedGroup.id));
    }
    setIsDeleteModalOpen(false);
    setSelectedGroup(null);
  };

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white font-display">
            Group Management
          </h1>
          <p className="mt-1 text-gray-500 dark:text-gray-400">
            Manage user groups and permissions ({groups.length} total)
          </p>
        </div>
        <Button onClick={handleCreate}>
          <Plus className="w-4 h-4" />
          Add Group
        </Button>
      </div>

      {/* Table */}
      <Table data={groups} columns={columns} searchable exportable pageSize={10} />

      {/* Form Modal */}
      <Modal
        isOpen={isFormModalOpen}
        onClose={() => { setIsFormModalOpen(false); setSelectedGroup(null); }}
        title={selectedGroup ? 'Edit Group' : 'Create Group'}
        size="md"
        closeOnClickOutside={false}
      >
        <GroupForm
          group={selectedGroup}
          onSubmit={handleFormSubmit}
          onCancel={() => { setIsFormModalOpen(false); setSelectedGroup(null); }}
        />
      </Modal>

      {/* Delete Modal */}
      <Modal
        isOpen={isDeleteModalOpen}
        onClose={() => { setIsDeleteModalOpen(false); setSelectedGroup(null); }}
        title="Confirm Delete"
        size="sm"
        closeOnClickOutside={false}
      >
        {selectedGroup && (
          <div className="space-y-4 text-center">
            <div className="flex items-center justify-center w-16 h-16 mx-auto bg-red-100 rounded-full">
              <Trash2 className="w-8 h-8 text-red-600" />
            </div>
            <p className="text-gray-500 dark:text-gray-400">
              Delete <strong>{selectedGroup.group_name}</strong>?
            </p>
            <div className="flex justify-center gap-3">
              <Button variant="secondary" onClick={() => { setIsDeleteModalOpen(false); setSelectedGroup(null); }}>Cancel</Button>
              <Button variant="danger" onClick={handleDeleteConfirm}>Delete</Button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
};
