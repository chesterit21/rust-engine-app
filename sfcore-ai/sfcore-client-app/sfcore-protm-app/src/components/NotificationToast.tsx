
import React, { useState, useEffect } from 'react';
import { X, CheckCircle, AlertCircle, Info, Loader2 } from 'lucide-react';

export type NotificationType = 'success' | 'error' | 'info' | 'loading';

export interface Notification {
    id: string;
    type: NotificationType;
    title: string;
    message: string;
    duration?: number;
}

interface NotificationToastProps {
    notifications: Notification[];
    removeNotification: (id: string) => void;
}

const icons = {
    success: <CheckCircle className="text-green-400" size={20} />,
    error: <X className="text-red-400" size={20} />,
    info: <Info className="text-blue-400" size={20} />,
    loading: <Loader2 className="text-purple-400 animate-spin" size={20} />
};

const backgrounds = {
    success: 'bg-gradient-to-r from-green-900/80 to-green-900/40 border-green-500/30',
    error: 'bg-gradient-to-r from-red-900/80 to-red-900/40 border-red-500/30',
    info: 'bg-gradient-to-r from-blue-900/80 to-blue-900/40 border-blue-500/30',
    loading: 'bg-gradient-to-r from-purple-900/80 to-purple-900/40 border-purple-500/30'
};

export const NotificationToast: React.FC<NotificationToastProps> = ({ notifications, removeNotification }) => {
    return (
        <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
            {notifications.map((notif) => (
                <div
                    key={notif.id}
                    className={`
                        pointer-events-auto
                        w-80 p-4 rounded-xl border backdrop-blur-md shadow-2xl
                        flex items-start gap-3
                        transition-all duration-300 ease-in-out transform translate-y-0 opacity-100
                        ${backgrounds[notif.type] || backgrounds.info}
                    `}
                    style={{ animation: 'slideIn 0.3s ease-out' }}
                >
                    <div className="mt-0.5 shrink-0">
                        {notif.type === 'error' ? <AlertCircle className="text-red-400" size={20} /> : icons[notif.type]}
                    </div>
                    
                    <div className="flex-1">
                        <h4 className="text-sm font-bold text-white mb-0.5 leading-tight">{notif.title}</h4>
                        <p className="text-xs text-slate-300 leading-relaxed">{notif.message}</p>
                        
                        {/* Simple Progress Bar for Loading */}
                        {notif.type === 'loading' && (
                            <div className="mt-2 h-1 w-full bg-black/20 rounded-full overflow-hidden">
                                <div 
                                    className="h-full bg-purple-500 animate-[progress_2s_ease-in-out_infinite]"
                                    style={{ width: '100%' }}
                                />
                            </div>
                        )}
                    </div>

                    <button 
                        onClick={() => removeNotification(notif.id)}
                        className="text-slate-400 hover:text-white transition"
                    >
                        <X size={16} />
                    </button>
                </div>
            ))}
            <style>{`
                @keyframes slideIn {
                    from { opacity: 0; transform: translateY(20px); }
                    to { opacity: 1; transform: translateY(0); }
                }
                @keyframes progress {
                    0% { transform: translateX(-100%); }
                    50% { transform: translateX(0); }
                    100% { transform: translateX(100%); }
                }
            `}</style>
        </div>
    );
};

// Hook for managing notifications easily
export const useNotifications = () => {
    const [notifications, setNotifications] = useState<Notification[]>([]);

    const addNotification = (type: NotificationType, title: string, message: string, duration = 4000) => {
        const id = Math.random().toString(36).substring(7);
        setNotifications(prev => [...prev, { id, type, title, message, duration }]);

        if (type !== 'loading') {
            setTimeout(() => {
                removeNotification(id);
            }, duration);
        }
        return id;
    };

    const removeNotification = (id: string) => {
        setNotifications(prev => prev.filter(n => n.id !== id));
    };

    return { notifications, addNotification, removeNotification };
};
