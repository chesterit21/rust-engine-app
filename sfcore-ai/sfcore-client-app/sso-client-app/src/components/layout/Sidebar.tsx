import React from 'react';
import { NavLink } from 'react-router-dom';
import {
  LayoutDashboard,
  Users,
  UserCog,
  Menu as MenuIcon,
  Building2,
  Shield,
  Settings,
  X,
} from 'lucide-react';
import clsx from 'clsx';

interface SidebarProps {
  isOpen: boolean;
  mobileOpen: boolean;
  onMobileClose: () => void;
}

const menuItems = [
  { icon: LayoutDashboard, label: 'Dashboard', path: '/' },
  { icon: Users, label: 'Users', path: '/users' },
  { icon: UserCog, label: 'Groups', path: '/groups' },
  { icon: MenuIcon, label: 'Menus', path: '/menus' },
  { icon: Building2, label: 'Tenants', path: '/tenants' },
  { icon: Shield, label: 'Permissions', path: '/permissions' },
  { icon: Settings, label: 'Settings', path: '/settings' },
];

export const Sidebar: React.FC<SidebarProps> = ({
  isOpen,
  mobileOpen,
  onMobileClose,
}) => {
  return (
    <>
      {/* Mobile Overlay */}
      {mobileOpen && (
        <div
          className="fixed inset-0 bg-black/50 z-40 lg:hidden"
          onClick={onMobileClose}
        />
      )}

      {/* Sidebar */}
      <aside
        className={clsx(
          'fixed top-0 left-0 h-full z-50',
          'bg-white dark:bg-dark-light border-r border-gray-200 dark:border-dark-lighter',
          'transition-all duration-300 ease-in-out',
          // Desktop
          'hidden lg:block',
          isOpen ? 'lg:w-64' : 'lg:w-20',
          // Mobile
          mobileOpen && 'block w-64'
        )}
      >
        {/* Logo */}
        <div className="h-16 flex items-center justify-between px-4 border-b border-gray-200 dark:border-dark-lighter">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-gradient-to-br from-primary-500 to-primary-600 rounded-xl flex items-center justify-center">
              <Shield className="w-6 h-6 text-white" />
            </div>
            {(isOpen || mobileOpen) && (
              <span className="font-display font-bold text-lg text-gray-900 dark:text-white">
                SSO Admin
              </span>
            )}
          </div>
          {mobileOpen && (
            <button onClick={onMobileClose} className="lg:hidden">
              <X className="w-6 h-6 text-gray-500" />
            </button>
          )}
        </div>

        {/* Navigation */}
        <nav className="mt-4 px-3">
          <ul className="space-y-1">
            {menuItems.map((item) => (
              <li key={item.path}>
                <NavLink
                  to={item.path}
                  className={({ isActive }) =>
                    clsx(
                      'flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors',
                      'text-gray-600 dark:text-gray-400',
                      'hover:bg-gray-100 dark:hover:bg-dark-lighter',
                      isActive && 'bg-primary-50 text-primary-600 dark:bg-primary-900/20 dark:text-primary-400 font-medium'
                    )
                  }
                  onClick={onMobileClose}
                >
                  <item.icon className="w-5 h-5 flex-shrink-0" />
                  {(isOpen || mobileOpen) && (
                    <span className="truncate">{item.label}</span>
                  )}
                </NavLink>
              </li>
            ))}
          </ul>
        </nav>
      </aside>
    </>
  );
};
