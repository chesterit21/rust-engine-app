import { writable } from 'svelte/store';

export const notifications = writable([]);

export function addNotification(type, message, duration = 5000) {
    const id = Date.now() + Math.random();
    notifications.update(n => [...n, { id, type, message }]);
    setTimeout(() => removeNotification(id), duration);
}

export function removeNotification(id) {
    notifications.update(n => n.filter(note => note.id !== id));
}
