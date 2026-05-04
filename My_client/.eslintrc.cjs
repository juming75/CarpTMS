module.exports = {
  root: true,
  env: {
    browser: true,
    node: true,
    es2024: true,
  },
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 2024,
    sourceType: 'module',
    extraFileExtensions: ['.vue'],
  },
  plugins: ['vue', '@typescript-eslint'],
  extends: [
    'eslint:recommended',
    'plugin:vue/vue3-recommended',
    'plugin:@typescript-eslint/recommended',
    '@vue/eslint-config-typescript',
    '@vue/eslint-config-prettier',
  ],
  rules: {
    'no-console': ['warn', { allow: ['warn', 'error'] }],
    'no-unused-vars': 'warn',
    '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
    '@typescript-eslint/no-explicit-any': 'warn',
    '@typescript-eslint/no-unsafe-assignment': 'warn',
    '@typescript-eslint/no-unsafe-member-access': 'warn',
    '@typescript-eslint/no-floating-promises': 'error',
    '@typescript-eslint/consistent-type-definitions': ['error', 'interface'],
    'vue/require-default-prop': 'off',
    'vue/multi-word-component-names': 'off',
  },
  overrides: [
    {
      files: ['*.ts', '*.tsx'],
      rules: {},
    },
    {
      files: ['*.vue'],
      rules: {},
    },
  ],
};
