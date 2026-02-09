
import { useEffect, useState } from 'react';
import { api } from '../api/client';
import { Plus, Edit2, Trash2, ChevronLeft, ChevronRight, ArrowUpDown, ArrowUp, ArrowDown, Search } from 'lucide-react';
import TechStackFormModal from '../components/TechStackFormModal';

const ITEMS_PER_PAGE = 10;

const TechStackPage = () => {
  const [data, setData] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<any | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  
  // Sort state
  const [sortConfig, setSortConfig] = useState<{ key: string, direction: 'asc' | 'desc' } | null>(null);

  const fetchData = () => {
    setLoading(true);
    // Fetch all for now as simple client-side pagination is requested/easier
    // Real-world would use query params like ?page=1&limit=10
    api.get('/tech-stacks')
      .then((res: any) => setData(res.data))
      .catch((err: any) => console.error(err))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    fetchData();
  }, []);

  // Sorting Logic
  const handleSort = (key: string) => {
    let direction: 'asc' | 'desc' = 'asc';
    if (sortConfig && sortConfig.key === key && sortConfig.direction === 'asc') {
      direction = 'desc';
    }
    setSortConfig({ key, direction });
  };

  // Filter Data
  const filteredData = data.filter(item => {
    if (!searchTerm) return true;
    const lowerTerm = searchTerm.toLowerCase();
    return (
        (item.name?.toLowerCase().includes(lowerTerm)) ||
        (item.stack_type?.toLowerCase().includes(lowerTerm)) ||
        (item.language?.toLowerCase().includes(lowerTerm)) ||
        (item.description?.toLowerCase().includes(lowerTerm))
    );
  });

  const sortedData = [...filteredData].sort((a, b) => {
    if (!sortConfig) return 0;
    
    // Handle stack_type alias if sorting by 'type'
    const key = sortConfig.key;
    const aValue = a[key]?.toString().toLowerCase() || '';
    const bValue = b[key]?.toString().toLowerCase() || '';

    if (aValue < bValue) {
      return sortConfig.direction === 'asc' ? -1 : 1;
    }
    if (aValue > bValue) {
      return sortConfig.direction === 'asc' ? 1 : -1;
    }
    return 0;
  });

  // Pagination Logic
  const totalPages = Math.ceil(sortedData.length / ITEMS_PER_PAGE);
  const paginatedData = sortedData.slice(
    (currentPage - 1) * ITEMS_PER_PAGE,
    currentPage * ITEMS_PER_PAGE
  );

  const handleSave = async (payload: any) => {
    try {
      if (Array.isArray(payload)) {
          // Bulk Create
          const promises = payload.map(item => api.post('/tech-stacks', item));
          await Promise.all(promises);
      } else {
          // Single Create/Update
          if (payload.id) {
            await api.put(`/tech-stacks/${payload.id}`, payload);
          } else {
            await api.post('/tech-stacks', payload);
          }
      }
      fetchData();
    } catch (err: any) {
      alert("Failed to save: " + (err.response?.data || err.message));
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm("Are you sure?")) return;
    try {
      await api.delete(`/tech-stacks/${id}`);
      fetchData();
    } catch (err) {
      alert("Failed to delete");
    }
  };
  
  const renderSortIcon = (key: string) => {
     if (sortConfig?.key !== key) return <ArrowUpDown size={14} className="ml-1 opacity-50" />;
     if (sortConfig.direction === 'asc') return <ArrowUp size={14} className="ml-1 text-neon-cyan" />;
     return <ArrowDown size={14} className="ml-1 text-neon-cyan" />;
  };

  return (
    <div className="p-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-white bg-gradient-to-r from-neon-cyan to-neon-teal bg-clip-text text-transparent">Tech Stacks</h1>
        
        <div className="flex items-center gap-4">
            <div className="relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" size={18} />
                <input 
                    type="text" 
                    placeholder="Search stack..." 
                    className="bg-dark-card border border-dark-border text-slate-200 pl-10 pr-4 py-2 rounded focus:outline-none focus:border-neon-cyan w-64"
                    value={searchTerm}
                    onChange={(e) => {
                        setSearchTerm(e.target.value);
                        setCurrentPage(1); // Reset to page 1 on search
                    }}
                />
            </div>
            <button 
              onClick={() => { setEditingItem(null); setIsModalOpen(true); }}
              className="flex items-center gap-2 bg-gradient-to-r from-neon-cyan to-neon-teal text-dark-bg font-bold px-4 py-2 rounded shadow-[0_0_15px_rgba(0,240,255,0.4)] hover:shadow-[0_0_25px_rgba(0,240,255,0.6)] transition cursor-pointer"
            >
              <Plus size={18} />
              <span>New Stack</span>
            </button>
        </div>
      </div>

      <div className="bg-dark-card rounded shadow overflow-hidden border-y border-white/5">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-white/5">
            <thead className="bg-dark-bg">
              <tr>
                <th 
                    className="px-6 py-3 text-left text-xs font-medium text-[#16f9f9] uppercase tracking-wider cursor-pointer hover:text-white transition group select-none"
                    onClick={() => handleSort('name')}
                >
                    <div className="flex items-center">
                        Name {renderSortIcon('name')}
                    </div>
                </th>
                <th 
                    className="px-6 py-3 text-left text-xs font-medium text-[#16f9f9] uppercase tracking-wider cursor-pointer hover:text-white transition group select-none"
                    onClick={() => handleSort('stack_type')}
                >
                    <div className="flex items-center">
                        Type {renderSortIcon('stack_type')}
                    </div>
                </th>
                <th 
                     className="px-6 py-3 text-left text-xs font-medium text-[#16f9f9] uppercase tracking-wider cursor-pointer hover:text-white transition group select-none"
                     onClick={() => handleSort('language')}
                >
                    <div className="flex items-center">
                        Language {renderSortIcon('language')}
                    </div>
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[#16f9f9] uppercase tracking-wider">Description</th>
                <th className="px-6 py-3 text-right text-xs font-medium text-[#16f9f9] uppercase tracking-wider">Actions</th>
              </tr>
            </thead>
            <tbody className="bg-dark-card divide-y divide-white/5">
              {loading ? (
                <tr>
                  <td colSpan={5} className="px-6 py-4 text-center text-sm text-slate-400">Loading...</td>
                </tr>
              ) : paginatedData.length === 0 ? (
                <tr>
                  <td colSpan={5} className="px-6 py-4 text-center text-sm text-slate-400">No records found.</td>
                </tr>
              ) : (
                paginatedData.map((item) => (
                  <tr key={item.id} className="hover:bg-dark-bg/50 transition">
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-white">{item.name}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-slate-300">
                      <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-neon-cyan/10 text-neon-cyan border border-neon-cyan/30">
                        {item.stack_type}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-slate-300">{item.language}</td>
                    <td className="px-6 py-4 text-sm text-slate-400 max-w-xs truncate">{item.description}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                      <button 
                        onClick={() => { setEditingItem(item); setIsModalOpen(true); }}
                        className="text-neon-cyan hover:text-white mr-4 transition cursor-pointer"
                        title="Edit"
                      >
                        <Edit2 size={16} />
                      </button>
                      <button 
                        onClick={() => handleDelete(item.id)}
                        className="text-red-500 hover:text-red-400 transition cursor-pointer"
                        title="Delete"
                      >
                        <Trash2 size={16} />
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
        
        {/* Pagination Controls */}
        {!loading && data.length > 0 && (
          <div className="bg-dark-bg px-4 py-3 flex items-center justify-between border-t border-dark-border sm:px-6">
             <div className="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between">
                <div>
                   <p className="text-sm text-slate-400">
                      Showing <span className="font-medium text-white">{(currentPage - 1) * ITEMS_PER_PAGE + 1}</span> to <span className="font-medium text-white">{Math.min(currentPage * ITEMS_PER_PAGE, data.length)}</span> of <span className="font-medium text-white">{data.length}</span> results
                   </p>
                </div>
                <div>
                   <nav className="relative z-0 inline-flex rounded-md shadow-sm -space-x-px" aria-label="Pagination">
                      <button
                        onClick={() => setCurrentPage(prev => Math.max(prev - 1, 1))}
                        disabled={currentPage === 1}
                        className="relative inline-flex items-center px-2 py-2 rounded-l-md border border-dark-border bg-dark-card text-sm font-medium text-slate-400 hover:bg-dark-bg disabled:opacity-50 cursor-pointer"
                      >
                         <ChevronLeft size={16} />
                      </button>
                      {/* Smart Page Numbers */}
                      {(() => {
                          const delta = 2;
                          const range = [];
                          for (let i = 1; i <= totalPages; i++) {
                            if (
                              i === 1 || 
                              i === totalPages || 
                              (i >= currentPage - delta && i <= currentPage + delta)
                            ) {
                              range.push(i);
                            }
                          }
                          
                          const rangeWithDots = [];
                          let l;
                          for (let i of range) {
                            if (l) {
                              if (i - l === 2) {
                                rangeWithDots.push(l + 1);
                              } else if (i - l !== 1) {
                                rangeWithDots.push('...');
                              }
                            }
                            rangeWithDots.push(i);
                            l = i;
                          }

                          return rangeWithDots.map((page, idx) => (
                              page === '...' ? (
                                <span key={`dots-${idx}`} className="relative inline-flex items-center px-4 py-2 border border-dark-border bg-dark-card text-sm font-medium text-slate-500">
                                  ...
                                </span>
                              ) : (
                                <button
                                  key={idx}
                                  onClick={() => setCurrentPage(Number(page))}
                                  className={`relative inline-flex items-center px-4 py-2 border text-sm font-medium cursor-pointer
                                    ${currentPage === page
                                      ? 'z-10 bg-neon-cyan/10 border-neon-cyan text-neon-cyan' 
                                      : 'bg-dark-card border-dark-border text-slate-400 hover:bg-dark-bg'
                                    }`}
                                >
                                  {page}
                                </button>
                              )
                          ));
                      })()}
                      <button
                        onClick={() => setCurrentPage(prev => Math.min(prev + 1, totalPages))}
                        disabled={currentPage === totalPages}
                        className="relative inline-flex items-center px-2 py-2 rounded-r-md border border-dark-border bg-dark-card text-sm font-medium text-slate-400 hover:bg-dark-bg disabled:opacity-50 cursor-pointer"
                      >
                        <ChevronRight size={16} />
                      </button>
                   </nav>
                </div>
             </div>
          </div>
        )}
      </div>

      {isModalOpen && (
        <TechStackFormModal 
          isOpen={isModalOpen}
          initialData={editingItem}
          onClose={() => setIsModalOpen(false)}
          onSubmit={handleSave}
        />
      )}
    </div>
  );
};

export default TechStackPage;
