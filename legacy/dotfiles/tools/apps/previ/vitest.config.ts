import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        environment: 'node', // Or 'jsdom' if you need a browser environment
        globals: true, // Enable global Vitest APIs
    },
});
