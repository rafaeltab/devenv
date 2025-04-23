import { describe, it, expect } from 'vitest';
import { Capability, Capabilities } from './capabilities';

describe('Capability', () => {
    it('should create a Capability instance with the correct name', () => {
        const capability = new Capability('test/capability');
        expect(capability.name).toBe('test/capability');
    });
});

describe('Capabilities', () => {
    it('should create a Capabilities instance with the correct capabilities', () => {
        const capability1 = new Capability('test/capability1');
        const capability2 = new Capability('test/capability2');
        const capabilities = new Capabilities([capability1, capability2]);

        expect(capabilities.list).toEqual([capability1, capability2]);
        expect(capabilities.length).toBe(2);
    });

    it('should check if a Capabilities instance has a specific capability (Capability instance)', () => {
        const capability1 = new Capability('test/capability1');
        const capability2 = new Capability('test/capability2');
        const capabilities = new Capabilities([capability1, capability2]);

        expect(capabilities.has(capability1)).toBe(true);
        expect(capabilities.has(new Capability('test/capability3'))).toBe(
            false
        );
    });

    it('should check if a Capabilities instance has a specific capability (string)', () => {
        const capability1 = new Capability('test/capability1');
        const capability2 = new Capability('test/capability2');
        const capabilities = new Capabilities([capability1, capability2]);

        expect(capabilities.has('test/capability1')).toBe(true);
        expect(capabilities.has('test/capability3')).toBe(false);
    });

    it('should check if a Capabilities instance has all capabilities from another Capabilities instance', () => {
        const capability1 = new Capability('test/capability1');
        const capability2 = new Capability('test/capability2');
        const capability3 = new Capability('test/capability3');
        const capabilities1 = new Capabilities([capability1, capability2]);
        const capabilities2 = new Capabilities([
            capability1,
            capability2,
            capability3,
        ]);

        expect(capabilities1.hasAll(capabilities1)).toBe(true);
        expect(capabilities2.hasAll(capabilities1)).toBe(true);
    });

    it('should check if a Capabilities instance has any capabilities from another Capabilities instance', () => {
        const capability1 = new Capability('test/capability1');
        const capability2 = new Capability('test/capability2');
        const capability3 = new Capability('test/capability3');
        const capabilities1 = new Capabilities([capability1, capability2]);
        const capabilities2 = new Capabilities([capability3]);

        expect(capabilities1.hasAny(capabilities1)).toBe(true);
        expect(capabilities1.hasAny(capabilities2)).toBe(false);
    });

    it('should merge multiple Capabilities instances into a single Capabilities instance', () => {
        const capability1 = new Capability('test/capability1');
        const capability2 = new Capability('test/capability2');
        const capability3 = new Capability('test/capability3');
        const capabilities1 = new Capabilities([capability1, capability2]);
        const capabilities2 = new Capabilities([capability3]);

        const mergedCapabilities = Capabilities.merge(
            capabilities1,
            capabilities2
        );

        expect(mergedCapabilities.list).toEqual([
            capability1,
            capability2,
            capability3,
        ]);
        expect(mergedCapabilities.length).toBe(3);
    });

    it('should create a Capabilities instance from an array of strings', () => {
        const capabilities = Capabilities.from([
            'test/capability1',
            'test/capability2',
        ]);

        expect(capabilities.list.map((c) => c.name)).toEqual([
            'test/capability1',
            'test/capability2',
        ]);
        expect(capabilities.length).toBe(2);
    });
});
