const eslintrc = {
  extends: [
    'airbnb',
    'plugin:prettier/recommended',
    'plugin:jest/recommended',
    // https://www.npmjs.com/package/@typescript-eslint/eslint-plugin
    'plugin:@typescript-eslint/recommended',
    // https://github.com/benmosher/eslint-plugin-import
    'plugin:import/errors',
    'plugin:import/warnings',
    'plugin:import/typescript',
    'plugin:markdown/recommended',
  ],
  env: {
    node: true,
    jasmine: true,
    jest: true,
    es6: true,
  },
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint', 'prettier', 'import', 'jest', 'markdown'],
  // https://github.com/typescript-eslint/typescript-eslint/issues/46#issuecomment-470486034
  overrides: [
    {
      files: ['*.ts', '*.tsx'],
      rules: {
        '@typescript-eslint/no-unused-vars': [2, { args: 'none' }],
      },
    },
  ],
  rules: {
    'no-console': 'off',
    'import/extensions': 'off',
    camelcase: [2, { properties: 'never' }],
  },
  settings: {
    jest: {
      version: 26,
    },
  },
};

module.exports = eslintrc;
