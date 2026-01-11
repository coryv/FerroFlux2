import js from '@eslint/js';
import ts from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';

/** @type {import('eslint').Linter.Config[]} */
export default [
    js.configs.recommended,
    ...ts.configs.recommended,
    ...svelte.configs['flat/recommended'],
    {
        languageOptions: {
            globals: {
                ...globals.browser,
                ...globals.node
            }
        }
    },
    {
        files: ['**/*.svelte', '**/*.ts', '**/*.svelte.ts'],
        languageOptions: {
            parserOptions: {
                parser: ts.parser
            }
        }
    },
    {
        ignores: ['build/', '.svelte-kit/', 'dist/', 'node_modules/']
    },
    {
        rules: {
            '@typescript-eslint/no-explicit-any': 'warn', // Downgrade to warn for now
            'svelte/no-unused-svelte-ignore': 'warn',
            'svelte/require-each-key': 'warn'
        }
    }
];
