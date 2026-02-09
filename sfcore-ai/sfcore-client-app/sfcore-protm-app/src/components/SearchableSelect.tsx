
import React, { useState, useEffect, useRef } from 'react';
import { ChevronDown, X, Search } from 'lucide-react';

interface Option {
    id: string;
    label: string;
    subLabel?: string;
}

interface SearchableSelectProps {
    label: string;
    placeholder?: string;
    options: Option[];
    value: string;
    onChange: (value: string) => void;
    disabled?: boolean;
}

const SearchableSelect: React.FC<SearchableSelectProps> = ({ 
    label, 
    placeholder = "Select...", 
    options, 
    value, 
    onChange,
    disabled = false
}) => {
    const [isOpen, setIsOpen] = useState(false);
    const [search, setSearch] = useState('');
    const dropdownRef = useRef<HTMLDivElement>(null);

    // Find selected option to display
    const selectedOption = options.find(o => o.id === value);

    // Initial load, maybe set search to selected label? No, keep empty search mostly.
    // User wants to see the selected item.

    // Close on click outside
    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setIsOpen(false);
                setSearch(''); // Clear search on close for next time
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    const filteredOptions = options.filter(option => 
        option.label.toLowerCase().includes(search.toLowerCase()) || 
        (option.subLabel && option.subLabel.toLowerCase().includes(search.toLowerCase()))
    );

    const handleSelect = (id: string) => {
        onChange(id);
        setIsOpen(false);
        setSearch('');
    };

    return (
        <div className="relative" ref={dropdownRef}>
            <label className="block text-sm font-medium text-slate-400 mb-1">{label}</label>
            
            {/* Trigger Button / Input */}
            <div 
                className={`w-full bg-dark-bg border border-white/10 rounded px-3 py-2 flex justify-between items-center cursor-pointer transition
                    ${isOpen ? 'border-neon-cyan ring-1 ring-neon-cyan' : 'hover:border-slate-500'}
                    ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
                `}
                onClick={() => !disabled && setIsOpen(!isOpen)}
            >
                <div className="flex-1 truncate text-white text-sm">
                    {selectedOption 
                        ? (
                            <span>
                                {selectedOption.label} 
                                {selectedOption.subLabel && <span className="text-slate-500 ml-2 text-xs">({selectedOption.subLabel})</span>}
                            </span>
                          )
                        : <span className="text-slate-500">{placeholder}</span>
                    }
                </div>
                <ChevronDown size={16} className={`text-slate-400 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
            </div>

            {/* Dropdown Menu */}
            {isOpen && (
                <div className="absolute z-50 w-full mt-1 bg-[#0f172a] border border-slate-700 rounded-md shadow-2xl max-h-60 overflow-hidden flex flex-col">
                    {/* Search Input */}
                    <div className="p-2 border-b border-slate-700 bg-slate-900">
                        <div className="relative">
                            <Search size={14} className="absolute left-2 top-2.5 text-slate-400" />
                            <input
                                autoFocus
                                type="text"
                                className="w-full bg-slate-800 border border-slate-600 rounded pl-8 pr-3 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-neon-cyan focus:ring-1 focus:ring-neon-cyan"
                                placeholder="Search..."
                                value={search}
                                onChange={(e) => setSearch(e.target.value)}
                                onClick={(e) => e.stopPropagation()} 
                            />
                        </div>
                    </div>

                    {/* Options List */}
                    <div className="overflow-y-auto flex-1 p-1">
                        {filteredOptions.length > 0 ? (
                            filteredOptions.map(option => (
                                <div 
                                    key={option.id}
                                    className={`px-3 py-2 text-sm rounded cursor-pointer transition flex justify-between items-center
                                        ${option.id === value 
                                            ? 'bg-neon-cyan/10 text-neon-cyan' 
                                            : 'text-slate-300 hover:bg-white/5 hover:text-white'}
                                    `}
                                    onClick={() => handleSelect(option.id)}
                                >
                                    <span>{option.label}</span>
                                    {option.subLabel && <span className="text-xs text-slate-500">{option.subLabel}</span>}
                                </div>
                            ))
                        ) : (
                            <div className="px-3 py-4 text-center text-xs text-slate-500">
                                No options found.
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
};

export default SearchableSelect;
