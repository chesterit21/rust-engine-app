// ============================================================================
// Table Component with TanStack Table
// File: src/components/ui/Table.tsx
// Features: Search, Sort, Pagination, Export (PDF/CSV/Excel)
// ============================================================================

import React from 'react';
import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  flexRender,
  ColumnDef,
  SortingState,
} from '@tanstack/react-table';
import { 
  ChevronLeft, 
  ChevronRight, 
  ChevronsLeft, 
  ChevronsRight,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  Search,
  FileText,
  FileSpreadsheet,
  File
} from 'lucide-react';
import clsx from 'clsx';

interface TableProps<T> {
  data: T[];
  columns: ColumnDef<T, unknown>[];
  searchable?: boolean;
  exportable?: boolean;
  onRowClick?: (row: T) => void;
  pageSize?: number;
}

export function Table<T>({ 
  data, 
  columns, 
  searchable = true, 
  exportable = true,
  onRowClick,
  pageSize = 10
}: TableProps<T>) {
  const [sorting, setSorting] = React.useState<SortingState>([]);
  const [globalFilter, setGlobalFilter] = React.useState('');

  const table = useReactTable({
    data,
    columns,
    state: {
      sorting,
      globalFilter,
    },
    onSortingChange: setSorting,
    onGlobalFilterChange: setGlobalFilter,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    initialState: {
      pagination: {
        pageSize,
      },
    },
  });

  // Export functions
  const exportToCSV = () => {
    const headers = columns.map((col) => {
      const header = col.header;
      return typeof header === 'string' ? header : col.id || '';
    }).filter(Boolean);
    
    const rows = table.getRowModel().rows.map(row => 
      row.getVisibleCells().map(cell => {
        const value = cell.getValue();
        return typeof value === 'string' ? `"${value.replace(/"/g, '""')}"` : value;
      }).join(',')
    );
    
    const csv = [headers.join(','), ...rows].join('\n');
    downloadFile(csv, 'export.csv', 'text/csv');
  };

  const exportToExcel = () => {
    // Simple Excel XML format
    const headers = columns.map((col) => {
      const header = col.header;
      return typeof header === 'string' ? header : col.id || '';
    }).filter(Boolean);
    
    let xml = '<?xml version="1.0"?><?mso-application progid="Excel.Sheet"?>';
    xml += '<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet">';
    xml += '<Worksheet ss:Name="Sheet1"><Table>';
    
    // Headers
    xml += '<Row>' + headers.map(h => `<Cell><Data ss:Type="String">${h}</Data></Cell>`).join('') + '</Row>';
    
    // Data
    table.getRowModel().rows.forEach(row => {
      xml += '<Row>';
      row.getVisibleCells().forEach(cell => {
        const value = cell.getValue();
        xml += `<Cell><Data ss:Type="String">${value ?? ''}</Data></Cell>`;
      });
      xml += '</Row>';
    });
    
    xml += '</Table></Worksheet></Workbook>';
    downloadFile(xml, 'export.xls', 'application/vnd.ms-excel');
  };

  const exportToPDF = () => {
    // Simple HTML to print/PDF
    const headers = columns.map((col) => {
      const header = col.header;
      return typeof header === 'string' ? header : col.id || '';
    }).filter(Boolean);
    
    let html = `
      <!DOCTYPE html>
      <html>
      <head>
        <title>Export</title>
        <style>
          body { font-family: Arial, sans-serif; padding: 20px; }
          table { width: 100%; border-collapse: collapse; margin-top: 20px; }
          th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
          th { background: #f5f5f5; font-weight: 600; }
          tr:nth-child(even) { background: #fafafa; }
        </style>
      </head>
      <body>
        <h1>Data Export</h1>
        <p>Generated: ${new Date().toLocaleString()}</p>
        <table>
          <thead>
            <tr>${headers.map(h => `<th>${h}</th>`).join('')}</tr>
          </thead>
          <tbody>
    `;
    
    table.getRowModel().rows.forEach(row => {
      html += '<tr>';
      row.getVisibleCells().forEach(cell => {
        html += `<td>${cell.getValue() ?? ''}</td>`;
      });
      html += '</tr>';
    });
    
    html += '</tbody></table></body></html>';
    
    const printWindow = window.open('', '_blank');
    if (printWindow) {
      printWindow.document.write(html);
      printWindow.document.close();
      printWindow.print();
    }
  };

  const downloadFile = (content: string, filename: string, mimeType: string) => {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-4">
      {/* Search & Export Controls */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        {searchable && (
          <div className="relative w-full sm:w-80">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
            <input
              type="text"
              value={globalFilter ?? ''}
              onChange={(e) => setGlobalFilter(e.target.value)}
              placeholder="Search all columns..."
              className="w-full pl-10 pr-4 py-2.5 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent dark:bg-dark-lighter dark:border-dark-lighter dark:text-white"
            />
          </div>
        )}

        {exportable && (
          <div className="flex gap-2">
            <button
              onClick={exportToPDF}
              className="flex items-center gap-2 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm font-medium"
            >
              <FileText className="w-4 h-4" />
              PDF
            </button>
            <button
              onClick={exportToCSV}
              className="flex items-center gap-2 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors text-sm font-medium"
            >
              <File className="w-4 h-4" />
              CSV
            </button>
            <button
              onClick={exportToExcel}
              className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
            >
              <FileSpreadsheet className="w-4 h-4" />
              Excel
            </button>
          </div>
        )}
      </div>

      {/* Table */}
      <div className="overflow-x-auto rounded-lg border border-gray-200 dark:border-dark-lighter bg-white dark:bg-dark-light">
        <table className="w-full">
          <thead className="bg-gray-50 dark:bg-dark-lighter">
            {table.getHeaderGroups().map(headerGroup => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map(header => (
                  <th
                    key={header.id}
                    className={clsx(
                      'px-6 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider',
                      header.column.getCanSort() && 'cursor-pointer hover:bg-gray-100 dark:hover:bg-dark-light select-none'
                    )}
                    onClick={header.column.getToggleSortingHandler()}
                  >
                    <div className="flex items-center gap-2">
                      {flexRender(
                        header.column.columnDef.header,
                        header.getContext()
                      )}
                      {header.column.getCanSort() && (
                        <span className="text-gray-400">
                          {{
                            asc: <ArrowUp className="w-4 h-4" />,
                            desc: <ArrowDown className="w-4 h-4" />,
                          }[header.column.getIsSorted() as string] ?? <ArrowUpDown className="w-4 h-4" />}
                        </span>
                      )}
                    </div>
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody className="divide-y divide-gray-200 dark:divide-dark-lighter">
            {table.getRowModel().rows.length === 0 ? (
              <tr>
                <td colSpan={columns.length} className="px-6 py-12 text-center text-gray-500">
                  No data found
                </td>
              </tr>
            ) : (
              table.getRowModel().rows.map(row => (
                <tr
                  key={row.id}
                  onClick={() => onRowClick?.(row.original)}
                  className={clsx(
                    'hover:bg-gray-50 dark:hover:bg-dark-lighter transition-colors',
                    onRowClick && 'cursor-pointer'
                  )}
                >
                  {row.getVisibleCells().map(cell => (
                    <td key={cell.id} className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-200">
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </td>
                  ))}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-4">
        <div className="text-sm text-gray-500 dark:text-gray-400">
          Showing {table.getState().pagination.pageIndex * table.getState().pagination.pageSize + 1} to{' '}
          {Math.min(
            (table.getState().pagination.pageIndex + 1) * table.getState().pagination.pageSize,
            table.getFilteredRowModel().rows.length
          )}{' '}
          of {table.getFilteredRowModel().rows.length} results
        </div>

        <div className="flex items-center gap-2">
          <select
            value={table.getState().pagination.pageSize}
            onChange={e => table.setPageSize(Number(e.target.value))}
            className="px-3 py-2 border border-gray-300 rounded-lg text-sm dark:bg-dark-lighter dark:border-dark-lighter dark:text-white"
          >
            {[10, 25, 50, 100].map(size => (
              <option key={size} value={size}>Show {size}</option>
            ))}
          </select>

          <div className="flex gap-1">
            <button
              onClick={() => table.setPageIndex(0)}
              disabled={!table.getCanPreviousPage()}
              className="p-2 border border-gray-300 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-100 dark:border-dark-lighter dark:hover:bg-dark-lighter"
            >
              <ChevronsLeft className="w-4 h-4" />
            </button>
            <button
              onClick={() => table.previousPage()}
              disabled={!table.getCanPreviousPage()}
              className="p-2 border border-gray-300 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-100 dark:border-dark-lighter dark:hover:bg-dark-lighter"
            >
              <ChevronLeft className="w-4 h-4" />
            </button>

            <span className="px-4 py-2 border border-gray-300 rounded-lg text-sm dark:border-dark-lighter">
              {table.getState().pagination.pageIndex + 1} / {table.getPageCount()}
            </span>

            <button
              onClick={() => table.nextPage()}
              disabled={!table.getCanNextPage()}
              className="p-2 border border-gray-300 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-100 dark:border-dark-lighter dark:hover:bg-dark-lighter"
            >
              <ChevronRight className="w-4 h-4" />
            </button>
            <button
              onClick={() => table.setPageIndex(table.getPageCount() - 1)}
              disabled={!table.getCanNextPage()}
              className="p-2 border border-gray-300 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-100 dark:border-dark-lighter dark:hover:bg-dark-lighter"
            >
              <ChevronsRight className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
