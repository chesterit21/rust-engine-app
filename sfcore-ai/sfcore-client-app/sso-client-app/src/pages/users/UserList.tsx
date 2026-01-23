// ============================================================================
// User List Page with Full CRUD
// File: src/pages/users/UserList.tsx
// Features: TanStack Table, Search, Sort, Pagination, Export, CRUD Modal
// ============================================================================

import React, { useState, useMemo } from 'react';
import { ColumnDef } from '@tanstack/react-table';
import { Plus, Edit, Trash2, UserCheck, UserX, Mail } from 'lucide-react';
import { Table } from '@/components/ui/Table';
import { Button } from '@/components/ui/Button';
import { Modal } from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';

// ============================================================================
// Types
// ============================================================================

interface User {
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

// ============================================================================
// Mock Data (30 users for demo)
// ============================================================================

const generateMockUsers = (): User[] => {
  const statuses: User['status_member'][] = ['active', 'wait_validation', 'suspended', 'blocked'];
  const names = [
    'John Doe', 'Jane Smith', 'Bob Wilson', 'Alice Brown', 'Charlie Davis',
    'Diana Miller', 'Edward Garcia', 'Fiona Martinez', 'George Anderson', 'Helen Thomas',
    'Ivan Jackson', 'Julia White', 'Kevin Harris', 'Laura Martin', 'Michael Lee',
    'Nancy Clark', 'Oscar Lewis', 'Patricia Walker', 'Quincy Hall', 'Rachel Allen',
    'Steven Young', 'Tina King', 'Ulysses Wright', 'Victoria Scott', 'William Green',
    'Xena Adams', 'Yusuf Baker', 'Zara Nelson', 'Aaron Hill', 'Bella Moore'
  ];

  return names.map((name, i) => ({
    id: `user-${String(i + 1).padStart(3, '0')}`,
    display_name: name,
    email: `${name.toLowerCase().replace(' ', '.')}@example.com`,
    status_member: statuses[i % statuses.length],
    is_active: i % 4 !== 3,
    is_login: i % 3 === 0,
    last_login: i % 3 === 0 ? new Date(Date.now() - i * 3600000).toISOString() : null,
    created_at: new Date(Date.now() - i * 86400000 * 7).toISOString(),
    updated_at: new Date(Date.now() - i * 86400000).toISOString(),
  }));
};

// ============================================================================
// Status Badge Component
// ============================================================================

const StatusBadge: React.FC<{ status: User['status_member'] }> = ({ status }) => {
  const styles = {
    active: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
    wait_validation: 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400',
    suspended: 'bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400',
    blocked: 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400',
  };

  const labels = {
    active: 'Active',
    wait_validation: 'Pending',
    suspended: 'Suspended',
    blocked: 'Blocked',
  };

  return (
    <span className={`px-2.5 py-1 rounded-full text-xs font-medium ${styles[status]}`}>
      {labels[status]}
    </span>
  );
};

// ============================================================================
// User Form Component (for Modal)
// ============================================================================

interface UserFormProps {
  user?: User | null;
  onSubmit: (data: Partial<User>) => void;
  onCancel: () => void;
}

const UserForm: React.FC<UserFormProps> = ({ user, onSubmit, onCancel }) => {
  const [formData, setFormData] = useState({
    display_name: user?.display_name || '',
    email: user?.email || '',
    password: '',
    status_member: user?.status_member || 'active',
    is_active: user?.is_active ?? true,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = () => {
    const newErrors: Record<string, string> = {};
    
    if (!formData.display_name.trim()) {
      newErrors.display_name = 'Display name is required';
    } else if (formData.display_name.length < 3) {
      newErrors.display_name = 'Display name must be at least 3 characters';
    }

    if (!formData.email.trim()) {
      newErrors.email = 'Email is required';
    } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email)) {
      newErrors.email = 'Invalid email format';
    }

    if (!user && !formData.password) {
      newErrors.password = 'Password is required for new users';
    } else if (formData.password && formData.password.length < 8) {
      newErrors.password = 'Password must be at least 8 characters';
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
        label="Display Name"
        placeholder="Enter display name"
        value={formData.display_name}
        onChange={(e) => setFormData({ ...formData, display_name: e.target.value })}
        error={errors.display_name}
      />

      <Input
        label="Email"
        type="email"
        placeholder="Enter email address"
        value={formData.email}
        onChange={(e) => setFormData({ ...formData, email: e.target.value })}
        error={errors.email}
        icon={<Mail className="w-4 h-4" />}
      />

      {!user && (
        <Input
          label="Password"
          type="password"
          placeholder="Enter password (min 8 characters)"
          value={formData.password}
          onChange={(e) => setFormData({ ...formData, password: e.target.value })}
          error={errors.password}
        />
      )}

      <div className="space-y-1">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Status
        </label>
        <select
          value={formData.status_member}
          onChange={(e) => setFormData({ ...formData, status_member: e.target.value as User['status_member'] })}
          className="w-full px-4 py-2.5 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 dark:bg-dark-lighter dark:border-dark-lighter dark:text-white"
        >
          <option value="active">Active</option>
          <option value="wait_validation">Pending Validation</option>
          <option value="suspended">Suspended</option>
          <option value="blocked">Blocked</option>
        </select>
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
          Account is active
        </label>
      </div>

      <div className="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-dark-lighter">
        <Button type="button" variant="secondary" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit">
          {user ? 'Update User' : 'Create User'}
        </Button>
      </div>
    </form>
  );
};

// ============================================================================
// Delete Confirmation Modal
// ============================================================================

interface DeleteConfirmProps {
  user: User;
  onConfirm: () => void;
  onCancel: () => void;
}

const DeleteConfirm: React.FC<DeleteConfirmProps> = ({ user, onConfirm, onCancel }) => (
  <div className="space-y-4">
    <div className="flex items-center justify-center w-16 h-16 mx-auto bg-red-100 rounded-full">
      <Trash2 className="w-8 h-8 text-red-600" />
    </div>
    <div className="text-center">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Delete User</h3>
      <p className="mt-2 text-gray-500 dark:text-gray-400">
        Are you sure you want to delete <strong>{user.display_name}</strong>?
        <br />This action cannot be undone.
      </p>
    </div>
    <div className="flex justify-center gap-3 pt-4">
      <Button variant="secondary" onClick={onCancel}>Cancel</Button>
      <Button variant="danger" onClick={onConfirm}>Delete User</Button>
    </div>
  </div>
);

// ============================================================================
// Main UserList Component
// ============================================================================

export const UserList: React.FC = () => {
  const [users, setUsers] = useState<User[]>(generateMockUsers);
  const [isFormModalOpen, setIsFormModalOpen] = useState(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);

  // Table columns definition
  const columns = useMemo<ColumnDef<User>[]>(() => [
    {
      accessorKey: 'display_name',
      header: 'User',
      cell: ({ row }) => (
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-gradient-to-br from-primary-400 to-primary-600 rounded-full flex items-center justify-center text-white font-medium">
            {row.original.display_name.split(' ').map(n => n[0]).join('').slice(0, 2)}
          </div>
          <div>
            <div className="font-medium text-gray-900 dark:text-white">
              {row.original.display_name}
            </div>
            <div className="text-sm text-gray-500">{row.original.email}</div>
          </div>
        </div>
      ),
    },
    {
      accessorKey: 'status_member',
      header: 'Status',
      cell: ({ getValue }) => <StatusBadge status={getValue() as User['status_member']} />,
    },
    {
      accessorKey: 'is_active',
      header: 'Active',
      cell: ({ getValue }) => (
        getValue() ? (
          <UserCheck className="w-5 h-5 text-green-500" />
        ) : (
          <UserX className="w-5 h-5 text-gray-400" />
        )
      ),
    },
    {
      accessorKey: 'last_login',
      header: 'Last Login',
      cell: ({ getValue }) => {
        const value = getValue() as string | null;
        if (!value) return <span className="text-gray-400">Never</span>;
        return new Date(value).toLocaleDateString('en-US', {
          month: 'short',
          day: 'numeric',
          year: 'numeric',
          hour: '2-digit',
          minute: '2-digit',
        });
      },
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
            onClick={(e) => {
              e.stopPropagation();
              handleEdit(row.original);
            }}
            className="p-1.5 hover:bg-gray-100 dark:hover:bg-dark-lighter rounded-lg transition-colors"
            title="Edit"
          >
            <Edit className="w-4 h-4 text-gray-500" />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleDelete(row.original);
            }}
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
  const handleCreate = () => {
    setSelectedUser(null);
    setIsFormModalOpen(true);
  };

  const handleEdit = (user: User) => {
    setSelectedUser(user);
    setIsFormModalOpen(true);
  };

  const handleDelete = (user: User) => {
    setSelectedUser(user);
    setIsDeleteModalOpen(true);
  };

  const handleFormSubmit = (data: Partial<User>) => {
    if (selectedUser) {
      // Update existing user
      setUsers(prev => prev.map(u => 
        u.id === selectedUser.id ? { ...u, ...data, updated_at: new Date().toISOString() } : u
      ));
    } else {
      // Create new user
      const newUser: User = {
        id: `user-${Date.now()}`,
        display_name: data.display_name!,
        email: data.email!,
        status_member: data.status_member || 'wait_validation',
        is_active: data.is_active ?? true,
        is_login: false,
        last_login: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      setUsers(prev => [newUser, ...prev]);
    }
    setIsFormModalOpen(false);
    setSelectedUser(null);
  };

  const handleDeleteConfirm = () => {
    if (selectedUser) {
      setUsers(prev => prev.filter(u => u.id !== selectedUser.id));
    }
    setIsDeleteModalOpen(false);
    setSelectedUser(null);
  };

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white font-display">
            User Management
          </h1>
          <p className="mt-1 text-gray-500 dark:text-gray-400">
            Manage all users in the system ({users.length} total)
          </p>
        </div>
        <Button onClick={handleCreate}>
          <Plus className="w-4 h-4" />
          Add User
        </Button>
      </div>

      {/* Table */}
      <Table
        data={users}
        columns={columns}
        searchable
        exportable
        pageSize={10}
      />

      {/* Create/Edit Modal */}
      <Modal
        isOpen={isFormModalOpen}
        onClose={() => {
          setIsFormModalOpen(false);
          setSelectedUser(null);
        }}
        title={selectedUser ? 'Edit User' : 'Create User'}
        size="md"
        closeOnClickOutside={false} // DISABLED as required
      >
        <UserForm
          user={selectedUser}
          onSubmit={handleFormSubmit}
          onCancel={() => {
            setIsFormModalOpen(false);
            setSelectedUser(null);
          }}
        />
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        isOpen={isDeleteModalOpen}
        onClose={() => {
          setIsDeleteModalOpen(false);
          setSelectedUser(null);
        }}
        title="Confirm Delete"
        size="sm"
        closeOnClickOutside={false}
      >
        {selectedUser && (
          <DeleteConfirm
            user={selectedUser}
            onConfirm={handleDeleteConfirm}
            onCancel={() => {
              setIsDeleteModalOpen(false);
              setSelectedUser(null);
            }}
          />
        )}
      </Modal>
    </div>
  );
};
